use proc_macro2::*;
use quote::*;
use syn::*;

pub(crate) fn expand_delete_item(
    partition_key: &(Ident, Type),
    sort_key: &Option<(Ident, Type)>,
    struct_name: &Ident,
) -> TokenStream {
    let trait_name = format_ident!("{}DeleteItem", struct_name);
    let client_name = format_ident!("{}Client", struct_name);
    let builder_name = format_ident!("{}DeleteItemBuilder", struct_name);
    let condition_token_name = format_ident!("{}ConditionToken", struct_name);
    let (partition_key_ident, partition_key_type) = partition_key;

    let client_trait = if let Some(sort_key) = sort_key {
        let (sort_key_ident, sort_key_type) = sort_key;
        quote! {
            pub trait #trait_name {
                fn delete(&self, pk: impl Into<#partition_key_type>, sk: impl Into<#sort_key_type>) -> #builder_name;
            }

            impl #trait_name for #client_name {
                fn delete(&self, pk: impl Into<#partition_key_type>, sk: impl Into<#sort_key_type>) -> #builder_name {
                    use ::std::iter::FromIterator;

                    let pk_attr: ::raiden::aws_sdk::types::AttributeValue = pk.into().into_attr();
                    let sk_attr: ::raiden::aws_sdk::types::AttributeValue = sk.into().into_attr();
                    let key_set: std::collections::HashMap<String, ::raiden::aws_sdk::types::AttributeValue> =
                        std::collections::HashMap::from_iter([
                            (stringify!(#partition_key_ident).to_owned(), pk_attr),
                            (stringify!(#sort_key_ident).to_owned(), sk_attr),
                        ]);

                    let mut builder = ::raiden::aws_sdk::operation::delete_item::DeleteItemInput::builder()
                        .set_key(Some(key_set))
                        .table_name(self.table_name());

                    #builder_name {
                        client: &self.client,
                        builder,
                    }
                }
            }
        }
    } else {
        quote! {
            pub trait #trait_name {
                fn delete(&self, key: impl Into<#partition_key_type>) -> #builder_name;
            }

            impl #trait_name for #client_name {
                fn delete(&self, key: impl Into<#partition_key_type>) -> #builder_name {
                    use ::std::iter::FromIterator;

                    let key_attr: ::raiden::aws_sdk::types::AttributeValue = key.into().into_attr();
                    let key_set: std::collections::HashMap<String, ::raiden::aws_sdk::types::AttributeValue> =
                        std::collections::HashMap::from_iter([
                            (stringify!(#partition_key_ident).to_owned(), key_attr),
                        ]);

                    let mut builder = ::raiden::aws_sdk::operation::delete_item::DeleteItemInput::builder()
                        .set_key(Some(key_set))
                        .table_name(self.table_name());

                    #builder_name {
                        client: &self.client,
                        builder,
                    }
                }
            }
        }
    };

    let api_call_token = super::api_call_token!("delete_item");
    let (call_inner_run, inner_run_args) = if cfg!(feature = "tracing") {
        (
            quote! {
                let table_name = self
                    .builder
                    .get_table_name()
                    .clone()
                    .expect("table name should be set");

                #builder_name::inner_run(&table_name, &self.client, self.builder).await?
            },
            quote! { table_name: &str, },
        )
    } else {
        (
            quote! { #builder_name::inner_run(&self.client, self.builder).await? },
            quote! {},
        )
    };

    quote! {
        #client_trait

        pub struct #builder_name<'a> {
            pub client: &'a ::raiden::Client,
            pub builder: ::raiden::aws_sdk::operation::delete_item::builders::DeleteItemInputBuilder,
        }

        impl<'a> #builder_name<'a> {
            pub fn raw_input(mut self, builder: ::raiden::aws_sdk::operation::delete_item::builders::DeleteItemInputBuilder) -> Self {
                self.builder = builder;
                self
            }

            pub fn condition(mut self, cond: impl ::raiden::condition::ConditionBuilder<#condition_token_name>) -> Self {
                let (cond_str, attr_names, attr_values) = cond.build();

                if !attr_names.is_empty() {
                    self.builder = self.builder
                        .set_expression_attribute_names(Some(attr_names));
                }

                if !attr_values.is_empty() {
                    self.builder = self.builder
                        .set_expression_attribute_values(Some(attr_values));
                }

                self.builder = self.builder.condition_expression(cond_str);
                self
            }

            pub async fn run(self) -> Result<(), ::raiden::RaidenError> {
                #call_inner_run;
                Ok(())
            }

            async fn inner_run(
                #inner_run_args
                client: &::raiden::Client,
                builder: ::raiden::aws_sdk::operation::delete_item::builders::DeleteItemInputBuilder,
            ) -> Result<(), ::raiden::RaidenError> {
                #api_call_token?;
                Ok(())
            }
        }
    }
}
