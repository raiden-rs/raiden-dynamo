use quote::*;
use syn::*;

pub(crate) fn expand_batch_delete(
    partition_key: &(Ident, Type),
    sort_key: &Option<(Ident, Type)>,
    struct_name: &Ident,
) -> proc_macro2::TokenStream {
    let trait_name = format_ident!("{}BatchDelete", struct_name);
    let client_name = format_ident!("{}Client", struct_name);
    let builder_name = format_ident!("{}BatchDeleteBuilder", struct_name);
    let (partition_key_ident, partition_key_type) = partition_key;

    let client_trait = if let Some(sort_key) = sort_key {
        let (sort_key_ident, sort_key_type) = sort_key;
        quote! {
            pub trait #trait_name {
                fn batch_delete(&self, keys: std::vec::Vec<(impl Into<#partition_key_type>, impl Into<#sort_key_type>)>) -> #builder_name;
            }

            impl #trait_name for #client_name {
                fn batch_delete(&self, keys: std::vec::Vec<(impl Into<#partition_key_type>, impl Into<#sort_key_type>)>) -> #builder_name {
                    let write_requests = {
                        let mut write_requests = vec![];
                        for (pk, sk) in keys.into_iter() {
                            let pk_attr_value = pk.into().into_attr();
                            let sk_attr_value = sk.into().into_attr();

                            let write_request = {
                                let mut write_request = ::raiden::WriteRequest::default();
                                let delete_request = ::raiden::DeleteRequest {
                                    key: vec![
                                        (stringify!(#partition_key_ident).to_string(), pk_attr_value),
                                        (stringify!(#sort_key_ident).to_string(), sk_attr_value)
                                    ].into_iter().collect(),
                                };
                                write_request.delete_request = Some(delete_request);
                                write_request
                            };

                            write_requests.push(write_request);
                        }

                        write_requests
                    };

                    #builder_name {
                        client: &self.client,
                        write_requests,
                        table_name: self.table_name(),
                        policy: self.retry_condition.strategy.policy(),
                        condition: &self.retry_condition,
                    }
                }
            }
        }
    } else {
        quote! {
            pub trait #trait_name {
                fn batch_delete(&self, keys: std::vec::Vec<impl Into<#partition_key_type>>) -> #builder_name;
            }

            impl #trait_name for #client_name {
                fn batch_delete(&self, keys: std::vec::Vec<impl Into<#partition_key_type>>) -> #builder_name {
                    let write_requests = {
                        let mut write_requests = vec![];
                        for pk in keys.into_iter() {
                            let pk_attr_value = pk.into().into_attr();

                            let write_request = {
                                let mut write_request = ::raiden::WriteRequest::default();
                                let delete_request = ::raiden::DeleteRequest {
                                    key: vec![
                                        (stringify!(#partition_key_ident).to_string(), pk_attr_value),
                                    ].into_iter().collect(),
                                };
                                write_request.delete_request = Some(delete_request);
                                write_request
                            };

                            write_requests.push(write_request);
                        }

                        write_requests
                    };

                    #builder_name {
                        client: &self.client,
                        write_requests,
                        table_name: self.table_name(),
                        policy: self.retry_condition.strategy.policy(),
                        condition: &self.retry_condition,
                    }
                }
            }
        }
    };

    let api_call_token = super::api_call_token!("batch_write_item");
    let (call_inner_run, inner_run_args) = if cfg!(feature = "tracing") {
        (
            quote! { #builder_name::inner_run(table_name, client, input).await },
            quote! { table_name: String, },
        )
    } else {
        (
            quote! { #builder_name::inner_run(client, input).await },
            quote! {},
        )
    };

    quote! {
        #client_trait

        pub struct #builder_name<'a> {
            pub client: &'a ::raiden::DynamoDbClient,
            pub write_requests: std::vec::Vec<::raiden::WriteRequest>,
            pub table_name: String,
            pub policy: ::raiden::Policy,
            pub condition: &'a ::raiden::retry::RetryCondition,
        }

        impl<'a> #builder_name<'a> {
            pub async fn run(self) -> Result<::raiden::batch_delete::BatchDeleteOutput, ::raiden::RaidenError> {
                let Self { client, mut write_requests, table_name, policy, condition } = self;
                let policy: ::raiden::RetryPolicy = policy.into();

                const RETRY: usize = 5;
                const MAX_ITEMS_PER_REQUEST: usize = 25;

                let mut exhausted = std::vec::Vec::new();
                while !write_requests.is_empty() {
                    let len = write_requests.len();
                    let start = len.saturating_sub(MAX_ITEMS_PER_REQUEST);
                    let req = write_requests.drain(start..).collect::<std::vec::Vec<_>>();
                    let unprocessed = ::raiden::retry::retry_batch_write_unprocessed_items(
                        req,
                        RETRY,
                        std::time::Duration::from_millis(50),
                        |requests| {
                            let table_name = table_name.clone();
                            let client = client.clone();
                            let policy = policy.clone();
                            async move {
                                let request_items = vec![(table_name.clone(), requests)]
                                    .into_iter()
                                    .collect::<std::collections::HashMap<_, _>>();
                                let input = ::raiden::BatchWriteItemInput {
                                    request_items,
                                    ..std::default::Default::default()
                                };
                                let response_table_name = table_name.clone();
                                let result = policy.retry_if(move || {
                                    let (table_name, client, input) =
                                        (table_name.clone(), client.clone(), input.clone());
                                    async move { #call_inner_run }
                                }, condition).await.map_err(std::boxed::Box::new)?;

                                Ok::<_, std::boxed::Box<::raiden::RaidenError>>(result.unprocessed_items
                                    .and_then(|mut items| items.remove(&response_table_name))
                                    .unwrap_or_default())
                            }
                        },
                    ).await.map_err(|err| *err)?;
                    exhausted.extend(unprocessed);
                }

                let unprocessed_items = exhausted
                    .into_iter()
                    .filter_map(|write_request| write_request.delete_request)
                    .collect::<std::vec::Vec<_>>();
                Ok(::raiden::batch_delete::BatchDeleteOutput {
                    consumed_capacity: None,
                    unprocessed_items,
                })
            }

            async fn inner_run(
                #inner_run_args
                client: ::raiden::DynamoDbClient,
                input: ::raiden::BatchWriteItemInput,
            ) -> Result<::raiden::BatchWriteItemOutput, ::raiden::RaidenError> {
                Ok(#api_call_token?)
            }
        }
    }
}
