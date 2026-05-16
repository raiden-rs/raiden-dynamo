use crate::rename::*;
use quote::*;
use syn::*;

pub(crate) fn expand_batch_put(
    struct_name: &Ident,
    fields: &FieldsNamed,
    rename_all_type: RenameAllType,
) -> proc_macro2::TokenStream {
    let item_input_name = format_ident!("{}PutItemInput", struct_name);
    let trait_name = format_ident!("{}BatchPut", struct_name);
    let client_name = format_ident!("{}Client", struct_name);
    let builder_name = format_ident!("{}BatchPutBuilder", struct_name);

    let input_items = {
        let insertion = fields.named.iter().map(|f| {
            let ident = &f.ident.clone().unwrap();
            let renamed = crate::finder::find_rename_value(&f.attrs);
            let attr_key = create_renamed(ident.to_string(), renamed, rename_all_type);
            if crate::finder::include_unary_attr(&f.attrs, "uuid") {
                quote! {
                    let id = #struct_name::gen();
                    input_item.insert(#attr_key.to_string(), id.into_attr());
                }
            } else {
                quote! {
                    let value = item.#ident.clone().into_attr();
                    if !::raiden::is_attr_value_empty(&value) {
                        input_item.insert(#attr_key.to_string(), value);
                    }
                }
            }
        });

        quote! {
            let mut input_item: std::collections::HashMap<String, ::raiden::AttributeValue> = std::collections::HashMap::new();
            #(#insertion)*
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
        pub trait #trait_name {
            fn batch_put(&self, items: std::vec::Vec<#item_input_name>) -> #builder_name;
        }

        impl #trait_name for #client_name {
            fn batch_put(&self, items: std::vec::Vec<#item_input_name>) -> #builder_name {
                let write_requests = {
                    let mut write_requests = vec![];
                    for item in items.into_iter() {
                        #input_items

                        let mut write_request = ::raiden::WriteRequest::default();
                        write_request.put_request = Some(::raiden::PutRequest {
                            item: input_item,
                        });
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

        pub struct #builder_name<'a> {
            pub client: &'a ::raiden::DynamoDbClient,
            pub write_requests: std::vec::Vec<::raiden::WriteRequest>,
            pub table_name: String,
            pub policy: ::raiden::Policy,
            pub condition: &'a ::raiden::retry::RetryCondition,
        }

        impl<'a> #builder_name<'a> {
            pub async fn run(self) -> Result<::raiden::batch_put::BatchPutOutput, ::raiden::RaidenError> {
                let Self { client, mut write_requests, table_name, policy, condition } = self;
                let policy: ::raiden::RetryPolicy = policy.into();

                const RETRY: usize = 5;
                const MAX_ITEMS_PER_REQUEST: usize = 25;

                for _ in 0..RETRY {
                    loop {
                        let len = write_requests.len();
                        if len == 0 {
                            break;
                        }

                        let start = len.saturating_sub(MAX_ITEMS_PER_REQUEST);
                        let end = std::cmp::min(len, start + MAX_ITEMS_PER_REQUEST);
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
                                let (table_name, client, input) = (t.clone(), c.clone(), i.clone());
                                async move { #call_inner_run }
                            }, condition).await?
                        };

                        let mut unprocessed_items = match result.unprocessed_items {
                            None => continue,
                            Some(unprocessed_items) if unprocessed_items.is_empty() => continue,
                            Some(unprocessed_items) => unprocessed_items,
                        };

                        let unprocessed_requests = unprocessed_items
                            .remove(&table_name)
                            .expect("request_items hashmap must have a value for the table name");
                        write_requests.extend(unprocessed_requests);
                    }
                }

                let unprocessed_items = write_requests
                    .into_iter()
                    .filter_map(|write_request| write_request.put_request)
                    .collect::<std::vec::Vec<_>>();
                Ok(::raiden::batch_put::BatchPutOutput {
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
