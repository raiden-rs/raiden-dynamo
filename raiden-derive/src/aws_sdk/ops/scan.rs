use convert_case::{Case, Casing};
use quote::*;

pub(crate) fn expand_scan(
    struct_name: &proc_macro2::Ident,
    _fields: &syn::FieldsNamed,
    _rename_all_type: crate::rename::RenameAllType,
    gsi_names: &[String],
) -> proc_macro2::TokenStream {
    let trait_name = format_ident!("{}Scan", struct_name);
    let client_name = format_ident!("{}Client", struct_name);
    let builder_name = format_ident!("{}ScanBuilder", struct_name);
    let projected_builder_name = format_ident!("{}ProjectedScanBuilder", struct_name);

    let filter_expression_token_name = format_ident!("{}FilterExpressionToken", struct_name);
    let gsi_methods = gsi_names.iter().map(|index_name| {
        let method_name = index_name.to_case(Case::Snake);
        let method_ident = if crate::helpers::is_reserved(&method_name) {
            format_ident!("r#{}", method_name)
        } else {
            format_ident!("{}", method_name)
        };
        quote! {
            pub fn #method_ident(mut self) -> Self {
                self.builder = self.builder.index_name(#index_name);
                self
            }
        }
    });
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

        /// A typed scan builder that decodes results into a projection item.
        ///
        /// This wrapper preserves the current scan state while changing the
        /// projection expression and output item type to `I`.
        pub struct #projected_builder_name<'a, I> {
            pub inner: #builder_name<'a>,
            pub _item: std::marker::PhantomData<fn() -> I>,
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
                    limit: None
                }
            }
        }

        impl<'a> #builder_name<'a> {
            #[deprecated(note = "use generated typed index method instead")]
            pub fn index(mut self, index: impl Into<String>) -> Self {
                self.builder = self.builder.index_name(index.into());
                self
            }

            #(#gsi_methods)*

            /// Switches the builder to an index projection type.
            ///
            /// This updates the projection expression and returns a typed
            /// wrapper whose `run()` method decodes items into `I`.
            pub fn project<I>(mut self) -> #projected_builder_name<'a, I>
            where
                I: ::raiden::RaidenIndexItem<#struct_name>,
            {
                self.builder = self.builder
                    .index_name(I::GSI_NAME)
                    .set_projection_expression(I::projection_expression())
                    .set_expression_attribute_names(I::attribute_names());
                #projected_builder_name {
                    inner: self,
                    _item: std::marker::PhantomData,
                }
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

            /// Runs the scan and decodes items into the builder output type.
            pub async fn run(self) -> Result<::raiden::scan::ScanOutput<#struct_name>, ::raiden::RaidenError> {
                self.run_inner::<#struct_name>().await
            }

            /// Runs the scan using the given index projection type.
            ///
            /// This is kept as a convenience wrapper for backward compatibility.
            pub async fn run_with<I>(self) -> Result<::raiden::scan::ScanOutput<I>, ::raiden::RaidenError>
            where
                I: ::raiden::RaidenIndexItem<#struct_name>,
            {
                self.project::<I>().run().await
            }

            async fn run_inner<I>(mut self) -> Result<::raiden::scan::ScanOutput<I>, ::raiden::RaidenError>
            where
                I: ::raiden::RaidenItem,
            {
                if let Some(token) = self.next_token {
                    self.builder = self.builder
                        .set_exclusive_start_key(Some(token.into_attr_values()?));
                }

                let mut items: Vec<I> = vec![];

                loop {
                    if let Some(limit) = self.limit {
                        self.builder = self.builder.limit(limit as i32);
                    }

                    let res = { #call_inner_run };

                    if let Some(res_items) = res.items {
                        for res_item in res_items.into_iter() {
                            items.push(I::from_item(res_item)?)
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

        impl<'a, I> #projected_builder_name<'a, I>
        where
            I: ::raiden::RaidenIndexItem<#struct_name>,
        {
            /// Replaces the projection item type while keeping scan state.
            pub fn project<J>(self) -> #projected_builder_name<'a, J>
            where
                J: ::raiden::RaidenIndexItem<#struct_name>,
            {
                self.inner.project::<J>()
            }

            /// Enables strongly consistent reads for the scan.
            pub fn consistent(mut self) -> Self {
                self.inner = self.inner.consistent();
                self
            }

            /// Applies a filter expression while preserving the projection type.
            pub fn filter(mut self, cond: impl ::raiden::filter_expression::FilterExpressionBuilder<#filter_expression_token_name>) -> Self {
                self.inner = self.inner.filter(cond);
                self
            }

            /// Sets the pagination token used to resume the scan.
            pub fn next_token(mut self, token: ::raiden::NextToken) -> Self {
                self.inner = self.inner.next_token(token);
                self
            }

            /// Limits the number of returned items.
            pub fn limit(mut self, limit: usize) -> Self {
                self.inner = self.inner.limit(limit);
                self
            }

            /// Runs the scan and decodes items into the projection item type.
            pub async fn run(self) -> Result<::raiden::scan::ScanOutput<I>, ::raiden::RaidenError> {
                self.inner.run_inner::<I>().await
            }

            /// Runs the scan with another projection item type.
            pub async fn run_with<J>(self) -> Result<::raiden::scan::ScanOutput<J>, ::raiden::RaidenError>
            where
                J: ::raiden::RaidenIndexItem<#struct_name>,
            {
                self.project::<J>().run().await
            }
        }
    }
}
