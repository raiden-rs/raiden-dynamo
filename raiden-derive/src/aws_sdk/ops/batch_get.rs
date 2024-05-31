use proc_macro2::*;
use quote::*;
use syn::*;

use crate::rename::*;

pub(crate) fn expand_batch_get(
    partition_key: &(Ident, Type),
    sort_key: &Option<(Ident, Type)>,
    struct_name: &Ident,
    fields: &FieldsNamed,
    rename_all_type: RenameAllType,
) -> proc_macro2::TokenStream {
    let trait_name = format_ident!("{}BatchGetItem", struct_name);
    let client_name = format_ident!("{}Client", struct_name);
    let builder_name = format_ident!("{}BatchGetItemBuilder", struct_name);
    let from_item = super::expand_attr_to_item(format_ident!("res_item"), fields, rename_all_type);
    let (partition_key_ident, partition_key_type) = partition_key;

    let builder_keys_type = if sort_key.is_none() {
        quote! { std::vec::Vec<::raiden::aws_sdk::types::AttributeValue> }
    } else {
        quote! { std::vec::Vec<(::raiden::aws_sdk::types::AttributeValue, ::raiden::aws_sdk::types::AttributeValue)> }
    };

    let insertion_attribute_name = fields.named.iter().map(|f| {
        let ident = &f.ident.clone().unwrap();
        let renamed = crate::finder::find_rename_value(&f.attrs);
        let result = create_renamed(ident.to_string(), renamed, rename_all_type);
        quote! {
            names.insert(
                format!("#{}", #result.clone()),
                #result.to_string(),
            );
        }
    });

    let builder_init = quote! {
        let names = {
            let mut names: ::raiden::AttributeNames = std::collections::HashMap::new();
            #(#insertion_attribute_name)*
            names
        };
        let projection_expression = Some(names.keys().map(|v| v.to_string()).collect::<Vec<String>>().join(", "));

        #builder_name {
            client: &self.client,
            table_name: self.table_name(),
            keys: key_attrs,
            attribute_names: Some(names),
            projection_expression
        }
    };

    let client_trait = if sort_key.is_none() {
        quote! {
            pub trait #trait_name {
                fn batch_get(&self, keys: std::vec::Vec<impl Into<#partition_key_type>>) -> #builder_name;
            }

            impl #trait_name for #client_name {
                fn batch_get(&self, keys: std::vec::Vec<impl Into<#partition_key_type>>) -> #builder_name {
                    let key_attrs = keys.into_iter().map(|v| v.into().into_attr()).collect();

                    #builder_init
                }
            }
        }
    } else {
        let (_, sort_key_type) = sort_key.clone().unwrap();
        quote! {
            pub trait #trait_name {
                fn batch_get(&self, keys: std::vec::Vec<(impl Into<#partition_key_type>, impl Into<#sort_key_type>)>) -> #builder_name;
            }

            impl #trait_name for #client_name {
                fn batch_get(&self, keys: std::vec::Vec<(impl Into<#partition_key_type>, impl Into<#sort_key_type>)>) -> #builder_name {
                    let key_attrs = keys.into_iter().map(|(pk, sk)| (pk.into().into_attr(), sk.into().into_attr())).collect();

                    #builder_init
                }
            }
        }
    };

    let convert_to_external_proc = if let Some(sort_key) = sort_key {
        let (sort_key_ident, _sort_key_type) = sort_key;
        quote! {
            for (pk_attr, sk_attr) in keys.into_iter() {
                let key_val: std::collections::HashMap<String, ::raiden::aws_sdk::types::AttributeValue> = ::std::collections::HashMap::from_iter([
                    (stringify!(#partition_key_ident).to_owned(), pk_attr),
                    (stringify!(#sort_key_ident).to_owned(), sk_attr),
                ]);

                item_builder = item_builder.keys(key_val);
            }
        }
    } else {
        quote! {
            for key_attr in keys.into_iter() {
                let key_val: std::collections::HashMap<String, ::raiden::aws_sdk::types::AttributeValue> = ::std::collections::HashMap::from_iter([
                    (stringify!(#partition_key_ident).to_owned(), key_attr),
                ]);

                item_builder = item_builder.keys(key_val);
            }
        }
    };

    let api_call_token = super::api_call_token!("batch_get_item");
    let (call_inner_run, inner_run_args) = if cfg!(feature = "tracing") {
        (
            quote! { #builder_name::inner_run(&self.table_name, &self.client, builder).await? },
            quote! { table_name: &str, },
        )
    } else {
        (
            quote! { #builder_name::inner_run(&self.client, builder).await? },
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
            pub projection_expression: Option<String>
        }

        impl<'a> #builder_name<'a> {

            #![allow(clippy::field_reassign_with_default)]
            pub async fn run(mut self) -> Result<::raiden::batch_get::BatchGetOutput<#struct_name>, ::raiden::RaidenError> {
                use ::std::iter::FromIterator;

                let mut items: std::vec::Vec<#struct_name> = vec![];
                let mut unprocessed_keys = ::raiden::aws_sdk::types::KeysAndAttributes::builder()
                    .set_keys(Some(vec![]))
                    .build()
                    .expect("should be built");

                // TODO: for now set 5, however we should make it more flexible.
                let mut unprocessed_retry = 5;
                loop {
                    let unprocessed_key_len = unprocessed_keys.keys().len();
                    let mut item_builder = ::raiden::aws_sdk::types::KeysAndAttributes::builder()
                        .set_expression_attribute_names(self.attribute_names.clone())
                        .set_projection_expression(self.projection_expression.clone())
                        .set_keys(Some(unprocessed_keys.keys));

                    if unprocessed_key_len < 100 {
                        let keys = self.keys.drain(0..std::cmp::min(100 - unprocessed_key_len, self.keys.len()));
                        #convert_to_external_proc
                    }

                    let builder = ::raiden::aws_sdk::operation::batch_get_item::BatchGetItemInput::builder()
                        .request_items(
                            self.table_name.to_string(),
                            item_builder.build().expect("should be built"),
                        );

                    let res = #call_inner_run;

                    if self.keys.is_empty() {
                        unprocessed_retry -= 1;
                    }

                    if let Some(res_responses) = res.responses {
                        let mut res_responses = res_responses;
                        if let Some(res_items) = (&mut res_responses).remove(&self.table_name) {
                            for res_item in res_items.into_iter() {
                                let mut res_item = res_item;
                                items.push(#struct_name {
                                    #(#from_item)*
                                })
                            }
                        } else {
                            return Err(::raiden::RaidenError::ResourceNotFound(format!("'{}' table not found or not active", &self.table_name)));
                        }
                    } else {
                        return Err(::raiden::RaidenError::ResourceNotFound("resource not found".to_owned()));
                    }

                    unprocessed_keys.keys = vec![];

                    if let Some(keys_by_table) = res.unprocessed_keys {
                        if let Some(keys_attrs) = keys_by_table.get(&self.table_name) {
                            unprocessed_keys.keys = keys_attrs.keys.clone();
                        }
                    }

                    if (
                        self.keys.is_empty() && unprocessed_keys.keys.is_empty()
                    ) || unprocessed_retry == 0
                    {
                            return Ok(::raiden::batch_get::BatchGetOutput {
                            consumed_capacity: res.consumed_capacity,
                            items,
                            unprocessed_keys: Some(unprocessed_keys),
                        })
                    }
                }
            }

            async fn inner_run(
                #inner_run_args
                client: &::raiden::Client,
                builder: ::raiden::aws_sdk::operation::batch_get_item::builders::BatchGetItemInputBuilder,
            ) -> Result<::raiden::aws_sdk::operation::batch_get_item::BatchGetItemOutput, ::raiden::RaidenError> {
                Ok(#api_call_token?)
            }
        }
    }
}
