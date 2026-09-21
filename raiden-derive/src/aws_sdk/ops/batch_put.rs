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
            let mut input_item: std::collections::HashMap<String, ::raiden::aws_sdk::types::AttributeValue> = std::collections::HashMap::new();
            #(#insertion)*
        }
    };

    let api_call_token = super::api_call_token!("batch_write_item");
    let (call_inner_run, inner_run_args) = if cfg!(feature = "tracing") {
        (
            quote! { #builder_name::inner_run(&table_name, &client, builder).await },
            quote! { table_name: &str, },
        )
    } else {
        (
            quote! { #builder_name::inner_run(&client, builder).await },
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

                        let put_request = ::raiden::aws_sdk::types::PutRequest::builder()
                            .set_item(Some(input_item))
                            .build()
                            .expect("should be built");

                        let write_request = ::raiden::aws_sdk::types::WriteRequest::builder()
                            .put_request(put_request)
                            .build();

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

        pub struct #builder_name<'a> {
            pub client: &'a ::raiden::Client,
            pub write_requests: ::std::vec::Vec<::raiden::aws_sdk::types::WriteRequest>,
            pub table_name: String,
        }

        impl<'a> #builder_name<'a> {
            pub async fn run(mut self) -> Result<::raiden::batch_put::BatchPutOutput, ::raiden::RaidenError> {
                const RETRY: usize = 5;
                const MAX_ITEMS_PER_REQUEST: usize = 25;

                let mut exhausted = std::vec::Vec::new();
                while !self.write_requests.is_empty() {
                    let len = self.write_requests.len();
                    let start = len.saturating_sub(MAX_ITEMS_PER_REQUEST);
                    let req = self.write_requests.drain(start..).collect::<std::vec::Vec<_>>();
                    let unprocessed = ::raiden::retry::retry_batch_write_unprocessed_items(
                        req,
                        RETRY,
                        std::time::Duration::from_millis(50),
                        |requests| {
                            let table_name = self.table_name.clone();
                            let client = self.client.clone();
                            async move {
                                let request_items = vec![(table_name.clone(), requests)]
                                    .into_iter()
                                    .collect::<std::collections::HashMap<_, _>>();
                                let builder = ::raiden::aws_sdk::operation::batch_write_item::BatchWriteItemInput::builder()
                                    .set_request_items(Some(request_items));
                                let result = #call_inner_run.map_err(std::boxed::Box::new)?;
                                Ok::<_, std::boxed::Box<::raiden::RaidenError>>(result.unprocessed_items
                                    .and_then(|mut items| items.remove(&table_name))
                                    .unwrap_or_default())
                            }
                        },
                    ).await.map_err(|err| *err)?;
                    exhausted.extend(unprocessed);
                }

                let unprocessed_items = exhausted
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
                client: &::raiden::Client,
                builder: ::raiden::operation::batch_write_item::builders::BatchWriteItemInputBuilder,
            ) -> Result<::raiden::operation::batch_write_item::BatchWriteItemOutput, ::raiden::RaidenError> {
                Ok(#api_call_token?)
            }
        }
    }
}
