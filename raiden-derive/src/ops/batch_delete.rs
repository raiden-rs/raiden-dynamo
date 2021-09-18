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

                    let request_items = {
                        let mut map = std::collections::HashMap::new();
                        map.insert(self.table_name(), write_requests);
                        map
                    };
                    let input = {
                        let mut input = ::raiden::BatchWriteItemInput::default();
                        input.request_items = request_items;
                        input
                    };

                    #builder_name {
                        client: &self.client,
                        input,
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

                    let request_items = {
                        let mut map = std::collections::HashMap::new();
                        map.insert(self.table_name(), write_requests);
                        map
                    };
                    let input = {
                        let mut input = ::raiden::BatchWriteItemInput::default();
                        input.request_items = request_items;
                        input
                    };

                    #builder_name {
                        client: &self.client,
                        input,
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
            pub input: ::raiden::BatchWriteItemInput,
            pub table_name: String,
        }

        impl<'a> #builder_name<'a> {
            pub async fn run(mut self) -> Result<::raiden::batch_delete::BatchDeleteOutput, ::raiden::RaidenError> {
                // TODO: set the number of retry to 5 for now, which should be made more flexible
                const RETRY: usize = 5;

                for _ in 0..RETRY {
                    // call a delete request
                    let result = self.client.batch_write_item(self.input).await?;
                    let unprocessed_items = match result.unprocessed_items {
                        None => {
                            let output = ::raiden::batch_delete::BatchDeleteOutput {
                                consumed_capacity: result.consumed_capacity,
                                unprocessed_items: vec![],
                            };
                            return Ok(output);
                        }
                        Some(unprocessed_items) => unprocessed_items,
                    };

                    let next_input = ::raiden::BatchWriteItemInput {
                        request_items: unprocessed_items,
                        ..std::default::Default::default()
                    };
                    self.input = next_input;
                }

                // when retry is done the specified times, treat it as success even if there are
                // still unprocessed items
                let unprocessed_items = self.input.request_items.remove(&self.table_name)
                    .expect("reqeust_items hashmap must have a value for the table name")
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
