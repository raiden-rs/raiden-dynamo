use quote::*;

pub(crate) fn expand_scan(
    struct_name: &proc_macro2::Ident,
    fields: &syn::FieldsNamed,
    rename_all_type: crate::rename::RenameAllType,
) -> proc_macro2::TokenStream {
    let trait_name = format_ident!("{}Scan", struct_name);
    let client_name = format_ident!("{}Client", struct_name);
    let builder_name = format_ident!("{}ScanBuilder", struct_name);

    let filter_expression_token_name = format_ident!("{}FilterExpressionToken", struct_name);
    let from_item = super::expand_attr_to_item(format_ident!("res_item"), fields, rename_all_type);
    let api_call_token = super::api_call_token!("scan");
    let (call_inner_run, inner_run_args) = if cfg!(feature = "tracing") {
        (
            quote! {
                let table_name = self.builder.get_table_name().clone().expect("table name should be set");
                #builder_name::inner_run(&table_name, &self.client, self.builder.clone()).await?
            },
            quote! { table_name: &str, },
        )
    } else {
        (
            quote! { #builder_name::inner_run(&self.client, self.builder.clone()).await? },
            quote! {},
        )
    };

    quote! {
        pub trait #trait_name {
            fn scan(&self) -> #builder_name;
        }

        pub struct #builder_name<'a> {
            pub client: &'a ::raiden::Client,
            pub builder: ::raiden::aws_sdk::operation::scan::builders::ScanInputBuilder,
            pub next_token: Option<::raiden::NextToken>,
            pub limit: Option<i64>
        }

        impl #trait_name for #client_name {
            #![allow(clippy::field_reassign_with_default)]
            fn scan(&self) -> #builder_name {
                let builder = ::raiden::aws_sdk::operation::scan::ScanInput::builder()
                    .table_name(self.table_name())
                    .set_projection_expression(self.projection_expression.clone())
                    .set_expression_attribute_names(self.attribute_names.clone());

                #builder_name {
                    client: &self.client,
                    builder,
                    next_token: None,
                    limit: None,
                }
            }
        }

        impl<'a> #builder_name<'a> {
            pub fn index(mut self, index: impl Into<String>) -> Self {
                self.builder = self.builder.index_name(index.into());
                self
            }

            pub fn consistent(mut self) -> Self {
                self.builder = self.builder.consistent_read(true);
                self
            }

            pub fn filter(mut self, cond: impl ::raiden::filter_expression::FilterExpressionBuilder<#filter_expression_token_name>) -> Self {
                let (cond_str, attr_names, attr_values) = cond.build();

                if !attr_values.is_empty() {
                    if let Some(v) = self.builder.get_expression_attribute_values().clone() {
                        self.builder = self.builder
                            .set_expression_attribute_values(Some(::raiden::merge_map(attr_values, v)));
                    } else {
                        self.builder = self.builder
                            .set_expression_attribute_values(Some(attr_values));
                    }
                }

                self.builder = self.builder.filter_expression(cond_str);
                self
            }

            pub fn next_token(mut self, token: ::raiden::NextToken) -> Self {
                self.next_token = Some(token);
                self
            }

            pub fn limit(mut self, limit: usize) -> Self {
                self.limit = Some(limit as i64);
                self
            }

            pub async fn run(mut self) -> Result<::raiden::scan::ScanOutput<#struct_name>, ::raiden::RaidenError> {
                if let Some(token) = self.next_token {
                    self.builder = self.builder
                        .set_exclusive_start_key(Some(token.into_attr_values()?));
                }

                let mut items: Vec<#struct_name> = vec![];

                loop {
                    if let Some(limit) = self.limit {
                        self.builder = self.builder.limit(limit as i32);
                    }

                    let res = { #call_inner_run };

                    if let Some(res_items) = res.items {
                        for res_item in res_items.into_iter() {
                            let mut res_item = res_item;
                            items.push(#struct_name {
                                #(#from_item)*
                            })
                        }
                    };

                    let scanned = res.scanned_count as i64;

                    let mut has_next = true;
                    if let Some(limit) = self.limit {
                        has_next = limit - scanned > 0;
                        self.limit = Some(limit - scanned);
                    }
                    if res.last_evaluated_key.is_none() || !has_next {
                        return Ok(::raiden::scan::ScanOutput {
                            consumed_capacity: res.consumed_capacity,
                            count: Some(res.count as i64),
                            items,
                            last_evaluated_key: res.last_evaluated_key,
                            scanned_count: Some(res.scanned_count as i64),
                        })
                    }

                    self.builder = self.builder
                        .set_exclusive_start_key(res.last_evaluated_key);
                }
            }

            async fn inner_run(
                #inner_run_args
                client: &::raiden::Client,
                builder: ::raiden::aws_sdk::operation::scan::builders::ScanInputBuilder,
            ) -> Result<::raiden::aws_sdk::operation::scan::ScanOutput, ::raiden::RaidenError> {
                Ok(#api_call_token?)
            }
        }
    }
}
