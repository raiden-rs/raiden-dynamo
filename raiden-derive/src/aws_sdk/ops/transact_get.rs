use proc_macro2::*;
use quote::*;
use syn::*;

pub(crate) fn expand_transact_get(
    partition_key: &(Ident, Type),
    sort_key: &Option<(Ident, Type)>,
    struct_name: &Ident,
    fields: &FieldsNamed,
    rename_all_type: crate::rename::RenameAllType,
) -> TokenStream {
    let trait_name = format_ident!("{}TransactGetItems", struct_name);
    let client_name = format_ident!("{}Client", struct_name);
    let builder_name = format_ident!("{}TransactGetItemsBuilder", struct_name);
    let from_item = super::expand_attr_to_item(format_ident!("res_item"), fields, rename_all_type);
    let (partition_key_ident, partition_key_type) = partition_key;

    let builder_keys_type = if sort_key.is_none() {
        quote! { std::vec::Vec<::raiden::aws_sdk::types::AttributeValue> }
    } else {
        quote! { std::vec::Vec<(::raiden::aws_sdk::types::AttributeValue, ::raiden::aws_sdk::types::AttributeValue)> }
    };

    let client_trait = if let Some(sort_key) = sort_key {
        let (_sort_key_ident, sort_key_type) = sort_key;
        quote! {
            pub trait #trait_name {
                fn transact_get(&self, keys: std::vec::Vec<(impl Into<#partition_key_type>, impl Into<#sort_key_type>)>) -> #builder_name;
            }

            impl #trait_name for #client_name {
                fn transact_get(&self, keys: std::vec::Vec<(impl Into<#partition_key_type>, impl Into<#sort_key_type>)>) -> #builder_name {
                    let keys = keys.into_iter().map(|(pk, sk)| (pk.into().into_attr(), sk.into().into_attr())).collect();

                    #builder_name {
                        client: &self.client,
                        table_name: self.table_name(),
                        keys,
                        attribute_names: self.attribute_names.clone(),
                        projection_expression: self.projection_expression.clone(),
                        policy: self.retry_condition.strategy.policy(),
                        condition: &self.retry_condition,
                    }
                }
            }
        }
    } else {
        quote! {
            pub trait #trait_name {
                fn transact_get(&self, keys: std::vec::Vec<impl Into<#partition_key_type>>) -> #builder_name;
            }

            impl #trait_name for #client_name {
                fn transact_get(&self, keys: std::vec::Vec<impl Into<#partition_key_type>>) -> #builder_name {
                    let keys = keys.into_iter().map(|key| key.into().into_attr()).collect();

                    #builder_name {
                        client: &self.client,
                        table_name: self.table_name(),
                        keys,
                        attribute_names: self.attribute_names.clone(),
                        projection_expression: self.projection_expression.clone(),
                        policy: self.retry_condition.strategy.policy(),
                        condition: &self.retry_condition,
                    }
                }
            }
        }
    };

    let push_gets = if let Some(sort_key) = sort_key {
        let (sort_key_ident, _) = sort_key;
        quote! {
            for (pk_attr, sk_attr) in self.keys.into_iter() {
                let key_set = ::std::collections::HashMap::from_iter([
                    (stringify!(#partition_key_ident).to_owned(), pk_attr),
                    (stringify!(#sort_key_ident).to_owned(), sk_attr),
                ]);
                let get = ::raiden::aws_sdk::types::Get::builder()
                    .table_name(self.table_name.clone())
                    .set_key(Some(key_set))
                    .set_expression_attribute_names(self.attribute_names.clone())
                    .set_projection_expression(self.projection_expression.clone())
                    .build()
                    .expect("should be built");
                transact_items.push(::raiden::aws_sdk::types::TransactGetItem::builder().get(get).build());
            }
        }
    } else {
        quote! {
            for key_attr in self.keys.into_iter() {
                let key_set = ::std::collections::HashMap::from_iter([
                    (stringify!(#partition_key_ident).to_owned(), key_attr),
                ]);
                let get = ::raiden::aws_sdk::types::Get::builder()
                    .table_name(self.table_name.clone())
                    .set_key(Some(key_set))
                    .set_expression_attribute_names(self.attribute_names.clone())
                    .set_projection_expression(self.projection_expression.clone())
                    .build()
                    .expect("should be built");
                transact_items.push(::raiden::aws_sdk::types::TransactGetItem::builder().get(get).build());
            }
        }
    };

    let api_call_token = super::api_call_token!("transact_get_items");
    let (call_inner_run, inner_run_args) = if cfg!(feature = "tracing") {
        (
            quote! { #builder_name::inner_run(table_name, client, builder).await },
            quote! { table_name: String, },
        )
    } else {
        (
            quote! { #builder_name::inner_run(client, builder).await },
            quote! {},
        )
    };

    quote! {
        #client_trait

        pub struct #builder_name<'a> {
            pub client: &'a ::raiden::Client,
            pub table_name: String,
            pub keys: #builder_keys_type,
            pub attribute_names: Option<::raiden::AttributeNames>,
            pub projection_expression: Option<String>,
            pub policy: ::raiden::Policy,
            pub condition: &'a ::raiden::retry::RetryCondition,
        }

        impl<'a> #builder_name<'a> {
            pub async fn run(self) -> Result<::raiden::transact_get::TransactGetOutput<#struct_name>, ::raiden::RaidenError> {
                let mut transact_items = vec![];
                #push_gets

                let builder = ::raiden::aws_sdk::operation::transact_get_items::TransactGetItemsInput::builder()
                    .set_transact_items(Some(transact_items));

                let policy: ::raiden::RetryPolicy = self.policy.into();
                let client = self.client;
                let table_name = self.table_name.clone();
                let res = policy.retry_if(move || {
                    let client = client.clone();
                    let builder = builder.clone();
                    let table_name = table_name.clone();
                    async move { #call_inner_run }
                }, self.condition).await?;

                let items = res.responses
                    .unwrap_or_default()
                    .into_iter()
                    .map(|response| {
                        match response.item {
                            Some(mut res_item) => Ok(Some(#struct_name {
                                #(#from_item)*
                            })),
                            None => Ok(None),
                        }
                    })
                    .collect::<Result<std::vec::Vec<_>, ::raiden::RaidenError>>()?;

                Ok(::raiden::transact_get::TransactGetOutput {
                    consumed_capacity: res.consumed_capacity,
                    items,
                })
            }

            async fn inner_run(
                #inner_run_args
                client: ::raiden::Client,
                builder: ::raiden::aws_sdk::operation::transact_get_items::builders::TransactGetItemsInputBuilder,
            ) -> Result<::raiden::aws_sdk::operation::transact_get_items::TransactGetItemsOutput, ::raiden::RaidenError> {
                Ok(#api_call_token?)
            }
        }
    }
}
