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

                // TODO: set the number of retry to 5 for now, which should be made more flexible
                const RETRY: usize = 5;
                const MAX_ITEMS_PER_REQUEST: usize = 25;

                for _ in 0..RETRY {
                    loop {
                        let len = write_requests.len();

                        // len == 0 means there are no items to be processed anymore
                        if len == 0 {
                            break;
                        }

                        let start = len.saturating_sub(MAX_ITEMS_PER_REQUEST);
                        let end = std::cmp::min(len, start + MAX_ITEMS_PER_REQUEST);
                        // take requests up to 25 from the request buffer
                        let req = write_requests.drain(start..end).collect::<std::vec::Vec<_>>();
                        let request_items = vec![(table_name.clone(), req)]
                            .into_iter()
                            .collect::<std::collections::HashMap<_, _>>();
                        let result = {
                            let t = table_name.clone();
                            let c = client.clone();
                            let i = ::raiden::BatchWriteItemInput {
                                request_items,
                                ..std::default::Default::default()
                            };

                            policy.retry_if(move || {
                                let (table_name, client, input)
                                    = (t.clone(), c.clone(), i.clone());
                                async move { #call_inner_run }
                            }, condition).await?
                        };

                        let mut unprocessed_items = match result.unprocessed_items {
                            None => {
                                // move on to the next iteration to check if there are unprocessed
                                // requests
                                continue;
                            }
                            Some(unprocessed_items) => {
                                if unprocessed_items.is_empty() {
                                    // move on to the next iteration to check if there are unprocessed
                                    // requests
                                    continue;
                                }

                                unprocessed_items
                            },
                        };

                        let unprocessed_requests = unprocessed_items
                            .remove(&table_name)
                            .expect("request_items hashmap must have a value for the table name");
                        // push unprocessed requests back to the request buffer
                        write_requests.extend(unprocessed_requests);
                    }
                }

                // when retry is done the specified times, treat it as success even if there are
                // still unprocessed items
                let unprocessed_items = write_requests
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
