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
                    }
                }
            }
        }
    };

    quote! {
        #client_trait

        pub struct #builder_name<'a> {
            pub client: &'a ::raiden::DynamoDbClient,
            pub write_requests: std::vec::Vec<::raiden::WriteRequest>,
            pub table_name: String,
        }

        impl<'a> #builder_name<'a> {
            pub async fn run(mut self) -> Result<::raiden::batch_delete::BatchDeleteOutput, ::raiden::RaidenError> {
                // TODO: set the number of retry to 5 for now, which should be made more flexible
                const RETRY: usize = 5;
                const MAX_ITEMS_PER_REQUEST: usize = 25;

                for _ in 0..RETRY {
                    loop {
                        let len = self.write_requests.len();

                        // len == 0 means there are no items to be processed anymore
                        if len == 0 {
                            break;
                        }

                        let start = len.saturating_sub(MAX_ITEMS_PER_REQUEST);
                        let end = std::cmp::min(len, start + MAX_ITEMS_PER_REQUEST);
                        // take requests up to 25 from the request buffer
                        let req = self.write_requests.drain(start..end).collect::<std::vec::Vec<_>>();
                        let request_items = vec![(self.table_name.clone(), req)]
                            .into_iter()
                            .collect::<std::collections::HashMap<_, _>>();
                        let input = ::raiden::BatchWriteItemInput {
                            request_items,
                            ..std::default::Default::default()
                        };

                        let result = self.client.batch_write_item(input).await?;
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
                            .remove(&self.table_name)
                            .expect("reqeust_items hashmap must have a value for the table name");
                        // push unprocessed requests back to the request buffer
                        self.write_requests.extend(unprocessed_requests);
                    }
                }

                // when retry is done the specified times, treat it as success even if there are
                // still unprocessed items
                let unprocessed_items = self.write_requests
                    .into_iter()
                    .filter_map(|write_request| write_request.delete_request)
                    .collect::<std::vec::Vec<_>>();
                Ok(::raiden::batch_delete::BatchDeleteOutput {
                    consumed_capacity: None,
                    unprocessed_items,
                })
            }
        }
    }
}
