use proc_macro2::*;
use quote::*;
use syn::*;

pub(crate) fn expand_get_item(
    partition_key: &(Ident, Type),
    sort_key: &Option<(Ident, Type)>,
    struct_name: &Ident,
    fields: &syn::FieldsNamed,
    rename_all_type: crate::rename::RenameAllType,
) -> TokenStream {
    let trait_name = format_ident!("{}GetItem", struct_name);
    let client_name = format_ident!("{}Client", struct_name);
    let builder_name = format_ident!("{}GetItemBuilder", struct_name);
    let from_item = super::expand_attr_to_item(format_ident!("res_item"), fields, rename_all_type);
    let (partition_key_ident, partition_key_type) = partition_key;

    let client_trait = if let Some(sort_key) = sort_key {
        let (sort_key_ident, sort_key_type) = sort_key;
        quote! {
            pub trait #trait_name {
                fn get(&self, pk: impl Into<#partition_key_type>, sk: impl Into<#sort_key_type>) -> #builder_name;
            }

            impl #trait_name for #client_name {
                fn get(&self, pk: impl Into<#partition_key_type>, sk: impl Into<#sort_key_type>) -> #builder_name {
                    use ::std::iter::FromIterator;

                    let pk_attr: ::raiden::AttributeValue = pk.into().into_attr();
                    let sk_attr: ::raiden::AttributeValue = sk.into().into_attr();
                    let key_set: std::collections::HashMap<String, ::raiden::AttributeValue> = std::collections::HashMap::from_iter([
                        (stringify!(#partition_key_ident).to_owned(), pk_attr),
                        (stringify!(#sort_key_ident).to_owned(), sk_attr),
                    ]);

                    let mut builder = ::raiden::GetItemInput::builder()
                        .set_expression_attribute_names(self.attribute_names.clone())
                        .set_key(Some(key_set))
                        .table_name(self.table_name());

                    if let Some(ref v) = self.projection_expression {
                        builder = builder.projection_expression(v.clone());
                    }

                    #builder_name {
                        client: &self.client,
                        builder,
                        policy: self.retry_condition.strategy.policy(),
                        condition: &self.retry_condition,
                    }
                }
            }
        }
    } else {
        quote! {
            pub trait #trait_name {
                fn get(&self, key: impl Into<#partition_key_type>) -> #builder_name;
            }

            impl #trait_name for #client_name {
                fn get(&self, key: impl Into<#partition_key_type>) -> #builder_name {
                    use ::std::iter::FromIterator;

                    let key_attr: ::raiden::AttributeValue = key.into().into_attr();
                    let key_set: std::collections::HashMap<String, ::raiden::AttributeValue> = std::collections::HashMap::from_iter([
                        (stringify!(#partition_key_ident).to_owned(), key_attr),
                    ]);

                    let mut builder = ::raiden::GetItemInput::builder()
                        .set_expression_attribute_names(self.attribute_names.clone())
                        .set_key(Some(key_set))
                        .table_name(self.table_name());

                    if let Some(ref v) = self.projection_expression {
                        builder = builder.projection_expression(v.clone());
                    }

                    #builder_name {
                        client: &self.client,
                        builder,
                        policy: self.retry_condition.strategy.policy(),
                        condition: &self.retry_condition,
                    }
                }
            }
        }
    };

    let api_call_token = super::api_call_token!("get_item");
    let (call_inner_run, inner_run_args) = if cfg!(feature = "tracing") {
        (
            quote! {
                let table_name = builder
                    .get_table_name()
                    .clone()
                    .expect("table name should be set");

                #builder_name::inner_run(table_name, client, builder).await
            },
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
            pub builder: ::raiden::GetItemInputBuilder,
            pub policy: ::raiden::Policy,
            pub condition: &'a ::raiden::retry::RetryCondition,
        }

        impl<'a> #builder_name<'a> {
            pub fn consistent(mut self) -> Self {
                self.builder = self.builder.consistent_read(true);
                self
            }

            pub async fn run(self) -> Result<::raiden::get::GetOutput<#struct_name>, ::raiden::RaidenError> {
                let policy: ::raiden::RetryPolicy = self.policy.into();
                let client = self.client;
                let builder = self.builder;
                policy.retry_if(move || {
                    let client = client.clone();
                    let builder = builder.clone();
                    async { #call_inner_run }
                }, self.condition).await
            }

            async fn inner_run(
                #inner_run_args
                client: ::raiden::Client,
                builder: ::raiden::GetItemInputBuilder,
            ) -> Result<::raiden::get::GetOutput<#struct_name>, ::raiden::RaidenError> {
                let res = #api_call_token?;
                if res.item.is_none() {
                    return Err(::raiden::RaidenError::ResourceNotFound("resource not found".to_owned()));
                };
                let mut res_item = res.item.unwrap();
                let item = #struct_name {
                    #(#from_item)*
                };
                Ok(::raiden::get::GetOutput {
                    item,
                    consumed_capacity: res.consumed_capacity,
                })
            }
        }
    }
}
