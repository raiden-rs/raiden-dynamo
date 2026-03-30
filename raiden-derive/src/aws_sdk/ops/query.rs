use convert_case::{Case, Casing};
use quote::*;

fn create_gsi_partition_token_name(
    struct_name: &proc_macro2::Ident,
    index_name: &str,
) -> proc_macro2::Ident {
    format_ident!(
        "{}{}PartitionKeyConditionToken",
        struct_name,
        index_name.to_case(Case::Pascal)
    )
}

fn create_gsi_sort_token_name(
    struct_name: &proc_macro2::Ident,
    index_name: &str,
    index: usize,
) -> proc_macro2::Ident {
    format_ident!(
        "{}{}Sort{}KeyConditionToken",
        struct_name,
        index_name.to_case(Case::Pascal),
        index + 1
    )
}

fn create_gsi_terminal_token_name(
    struct_name: &proc_macro2::Ident,
    index_name: &str,
) -> proc_macro2::Ident {
    format_ident!(
        "{}{}TerminalKeyConditionToken",
        struct_name,
        index_name.to_case(Case::Pascal)
    )
}

fn resolve_attr_name(
    fields: &syn::FieldsNamed,
    rename_all_type: crate::rename::RenameAllType,
    field_name: &str,
) -> String {
    let field = fields
        .named
        .iter()
        .find(|field| {
            field
                .ident
                .as_ref()
                .is_some_and(|ident| ident == field_name)
        })
        .unwrap_or_else(|| panic!("unknown field `{field_name}` for gsi partition_key"));

    crate::rename::create_renamed(
        field_name.to_owned(),
        crate::finder::find_rename_value(&field.attrs),
        rename_all_type,
    )
}

pub(crate) fn expand_query(
    struct_name: &proc_macro2::Ident,
    fields: &syn::FieldsNamed,
    rename_all_type: crate::rename::RenameAllType,
    gsi_names: &[String],
    gsi_definitions: &[crate::finder::GsiDefinition],
) -> proc_macro2::TokenStream {
    let trait_name = format_ident!("{}Query", struct_name);
    let client_name = format_ident!("{}Client", struct_name);
    let builder_name = format_ident!("{}QueryBuilder", struct_name);
    let projected_builder_name = format_ident!("{}ProjectedQueryBuilder", struct_name);
    let query_output_item = format_ident!("{}QueryOutput", struct_name);

    let filter_expression_token_name = format_ident!("{}FilterExpressionToken", struct_name);
    let key_condition_token_name = format_ident!("{}KeyConditionToken", struct_name);
    let gsi_tokens = gsi_definitions.iter().flat_map(|gsi| {
        let mut tokens = vec![];
        if gsi.partition_key.is_none() {
            return tokens;
        }

        let partition_token_name = create_gsi_partition_token_name(struct_name, &gsi.name);
        let terminal_token_name = create_gsi_terminal_token_name(struct_name, &gsi.name);
        tokens.push(quote! {
            pub struct #partition_token_name;
            impl ::raiden::key_condition::SupportsEqCondition for #partition_token_name {}
        });
        tokens.push(quote! {
            pub struct #terminal_token_name;
        });

        for index in 0..gsi.sort_keys.len() {
            let sort_token_name = create_gsi_sort_token_name(struct_name, &gsi.name, index);
            if index + 1 == gsi.sort_keys.len() {
                tokens.push(quote! {
                    pub struct #sort_token_name;
                    impl ::raiden::key_condition::SupportsEqCondition for #sort_token_name {}
                    impl ::raiden::key_condition::SupportsRangeCondition for #sort_token_name {}
                });
            } else {
                tokens.push(quote! {
                    pub struct #sort_token_name;
                    impl ::raiden::key_condition::SupportsEqCondition for #sort_token_name {}
                });
            }
        }

        tokens
    });
    let gsi_key_condition_methods = gsi_definitions.iter().flat_map(|gsi| {
        let mut methods = vec![];
        let Some(partition_key) = gsi.partition_key.as_ref() else {
            return methods;
        };
        let partition_token_name = create_gsi_partition_token_name(struct_name, &gsi.name);
        let terminal_token_name = create_gsi_terminal_token_name(struct_name, &gsi.name);
        let partition_next_token_name = if gsi.sort_keys.is_empty() {
            terminal_token_name.clone()
        } else {
            create_gsi_sort_token_name(struct_name, &gsi.name, 0)
        };

        let method_name = format!("{}_key_condition", gsi.name.to_case(Case::Snake));
        let method_ident = if crate::helpers::is_reserved(&method_name) {
            format_ident!("r#{}", method_name)
        } else {
            format_ident!("{}", method_name)
        };
        let attr_name = resolve_attr_name(fields, rename_all_type, partition_key);

        methods.push(quote! {
            pub fn #method_ident() -> ::raiden::KeyCondition<#partition_token_name, #partition_next_token_name> {
                ::raiden::KeyCondition {
                    attr: #attr_name.to_owned(),
                    _token: std::marker::PhantomData,
                    _next_token: std::marker::PhantomData,
                }
            }
        });

        if gsi.sort_keys.len() == 1 {
            let sort_key = &gsi.sort_keys[0];
            let sort_token_name = create_gsi_sort_token_name(struct_name, &gsi.name, 0);
            let method_name = format!("{}_sort_key_condition", gsi.name.to_case(Case::Snake));
            let method_ident = if crate::helpers::is_reserved(&method_name) {
                format_ident!("r#{}", method_name)
            } else {
                format_ident!("{}", method_name)
            };
            let attr_name = resolve_attr_name(fields, rename_all_type, sort_key);

            methods.push(quote! {
                pub fn #method_ident() -> ::raiden::KeyCondition<#sort_token_name, #terminal_token_name> {
                    ::raiden::KeyCondition {
                        attr: #attr_name.to_owned(),
                        _token: std::marker::PhantomData,
                        _next_token: std::marker::PhantomData,
                    }
                }
            });
        } else {
            for (index, sort_key) in gsi.sort_keys.iter().enumerate() {
                let sort_token_name = create_gsi_sort_token_name(struct_name, &gsi.name, index);
                let next_token_name = if index + 1 < gsi.sort_keys.len() {
                    create_gsi_sort_token_name(struct_name, &gsi.name, index + 1)
                } else {
                    terminal_token_name.clone()
                };
                let method_name = format!(
                    "{}_sort_key_condition_{}",
                    gsi.name.to_case(Case::Snake),
                    index + 1
                );
                let method_ident = if crate::helpers::is_reserved(&method_name) {
                    format_ident!("r#{}", method_name)
                } else {
                    format_ident!("{}", method_name)
                };
                let attr_name = resolve_attr_name(fields, rename_all_type, sort_key);

                methods.push(quote! {
                    pub fn #method_ident() -> ::raiden::KeyCondition<#sort_token_name, #next_token_name> {
                        ::raiden::KeyCondition {
                            attr: #attr_name.to_owned(),
                            _token: std::marker::PhantomData,
                            _next_token: std::marker::PhantomData,
                        }
                    }
                });
            }
        }

        methods
    });
    let gsi_methods = gsi_names.iter().map(|index_name| {
        let method_name = index_name.to_case(Case::Snake);
        let method_ident = if crate::helpers::is_reserved(&method_name) {
            format_ident!("r#{}", method_name)
        } else {
            format_ident!("{}", method_name)
        };
        let typed_token_name = gsi_definitions
            .iter()
            .find(|gsi| gsi.name == *index_name && gsi.partition_key.is_some())
            .map(|gsi| create_gsi_partition_token_name(struct_name, &gsi.name));

        if let Some(token_name) = typed_token_name {
            quote! {
                pub fn #method_ident(self) -> #builder_name<'a, #token_name> {
                    let Self {
                        client,
                        builder,
                        next_token,
                        limit,
                        policy,
                        condition,
                        ..
                    } = self;
                    #builder_name {
                        client,
                        builder: builder.index_name(#index_name),
                        next_token,
                        limit,
                        policy,
                        condition,
                        _token: std::marker::PhantomData::<fn() -> #token_name>,
                    }
                }
            }
        } else {
            quote! {
                pub fn #method_ident(mut self) -> Self {
                    self.builder = self.builder.index_name(#index_name);
                    self
                }
            }
        }
    });

    let api_call_token = super::api_call_token!("query");
    let (call_inner_run, inner_run_args) = if cfg!(feature = "tracing") {
        (
            quote! {
                let table_name = builder
                    .get_table_name()
                    .clone()
                    .expect("table name should be set");

                Self::inner_run(table_name, client, builder).await
            },
            quote! { table_name: String, },
        )
    } else {
        (quote! { Self::inner_run(client, builder).await }, quote! {})
    };

    quote! {
        pub trait #trait_name {
            fn query(&self) -> #builder_name<'_, #key_condition_token_name>;
        }

        pub struct #builder_name<'a, T = #key_condition_token_name> {
            pub client: &'a ::raiden::Client,
            pub builder: ::raiden::aws_sdk::operation::query::builders::QueryInputBuilder,
            pub next_token: Option<::raiden::NextToken>,
            pub limit: Option<i64>,
            pub policy: ::raiden::Policy,
            pub condition: &'a ::raiden::retry::RetryCondition,
            pub _token: std::marker::PhantomData<fn() -> T>,
        }

        /// A typed query builder that decodes results into a projection item.
        ///
        /// This wrapper preserves the current query state while changing the
        /// projection expression and output item type to `I`.
        pub struct #projected_builder_name<'a, T, I> {
            pub inner: #builder_name<'a, T>,
            pub _item: std::marker::PhantomData<fn() -> I>,
        }

        struct #query_output_item {
            consumed_capacity: Option<::raiden::aws_sdk::types::ConsumedCapacity>,
            count: Option<i64>,
            items: Option<Vec<::std::collections::HashMap<String, ::raiden::aws_sdk::types::AttributeValue>>>,
            last_evaluated_key: Option<::std::collections::HashMap<String, ::raiden::aws_sdk::types::AttributeValue>>,
            scanned_count: Option<i64>,
        }

        #(#gsi_tokens)*

        impl #trait_name for #client_name {

            #![allow(clippy::field_reassign_with_default)]
            fn query(&self) -> #builder_name<'_, #key_condition_token_name> {
                let builder = ::raiden::aws_sdk::operation::query::QueryInput::builder()
                    .table_name(self.table_name())
                    .set_projection_expression(self.projection_expression.clone())
                    .set_expression_attribute_names(self.attribute_names.clone());

                #builder_name {
                    client: &self.client,
                    builder,
                    next_token: None,
                    limit: None,
                    policy: self.retry_condition.strategy.policy(),
                    condition: &self.retry_condition,
                    _token: std::marker::PhantomData::<fn() -> #key_condition_token_name>,
                }
            }
        }

        impl #struct_name {
            #(#gsi_key_condition_methods)*
        }

        impl<'a, T> #builder_name<'a, T> {
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
            pub fn project<I>(mut self) -> #projected_builder_name<'a, T, I>
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

            pub fn next_token(mut self, token: ::raiden::NextToken) -> Self {
                self.next_token = Some(token);
                self
            }

            pub fn desc(mut self) -> Self {
                self.builder = self.builder.scan_index_forward(false);
                self
            }

            pub fn asc(mut self) -> Self {
                self.builder = self.builder.scan_index_forward(true);
                self
            }

            pub fn limit(mut self, limit: usize) -> Self {
                self.limit = Some(limit as i64);
                self
            }

            pub fn filter(mut self, cond: impl ::raiden::filter_expression::FilterExpressionBuilder<#filter_expression_token_name>) -> Self {
                let (cond_str, attr_names, attr_values) = cond.build();

                if !attr_names.is_empty() {
                    if let Some(v) = self.builder.get_expression_attribute_names().clone() {
                        self.builder = self.builder
                            .set_expression_attribute_names(Some(::raiden::merge_map(attr_names, v)));
                    } else {
                        self.builder = self.builder
                            .set_expression_attribute_names(Some(attr_names));
                    }
                }

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

            pub fn key_condition<U>(mut self, cond: impl ::raiden::key_condition::KeyConditionBuilder<T, U>) -> Self {
                let (cond_str, attr_names, attr_values) = cond.build();

                if !attr_names.is_empty() {
                    if let Some(v) = self.builder.get_expression_attribute_names().clone() {
                        self.builder = self
                            .builder
                            .set_expression_attribute_names(Some(::raiden::merge_map(attr_names, v)));
                    } else {
                        self.builder = self
                            .builder
                            .set_expression_attribute_names(Some(attr_names));
                    }
                }

                if !attr_values.is_empty() {
                    if let Some(v) = self.builder.get_expression_attribute_values().clone() {
                        self.builder = self
                            .builder
                            .set_expression_attribute_values(Some(::raiden::merge_map(attr_values, v)));
                    } else {
                        self.builder = self
                            .builder
                            .set_expression_attribute_values(Some(attr_values));
                    }
                }

                self.builder = self.builder.key_condition_expression(cond_str);
                self
            }

            /// Runs the query and decodes items into the builder output type.
            pub async fn run(self) -> Result<::raiden::query::QueryOutput<#struct_name>, ::raiden::RaidenError> {
                self.run_inner::<#struct_name>().await
            }

            /// Runs the query using the given index projection type.
            ///
            /// This is kept as a convenience wrapper for backward compatibility.
            pub async fn run_with<I>(self) -> Result<::raiden::query::QueryOutput<I>, ::raiden::RaidenError>
            where
                I: ::raiden::RaidenIndexItem<#struct_name>,
            {
                self.project::<I>().run().await
            }

            async fn run_inner<I>(mut self) -> Result<::raiden::query::QueryOutput<I>, ::raiden::RaidenError>
            where
                I: ::raiden::RaidenItem,
            {
                if let Some(token) = self.next_token {
                    self.builder = self.builder
                        .set_exclusive_start_key(Some(token.into_attr_values()?));
                    }

                let mut items: Vec<I> = vec![];
                let policy: ::raiden::RetryPolicy = self.policy.into();
                let client = self.client;

                loop {
                    if let Some(limit) = self.limit {
                        self.builder = self.builder.limit(limit as i32);
                    }

                    let builder = self.builder.clone();
                    let client = self.client.clone();

                    let res: #query_output_item = policy.retry_if(move || {
                        let builder = builder.clone();
                        let client = client.clone();
                        async { #call_inner_run }
                    }, self.condition).await?;

                    if let Some(res_items) = res.items {
                        for res_item in res_items.into_iter() {
                            items.push(I::from_item(res_item)?)
                        }
                    };

                    let scanned = res.scanned_count.unwrap_or(0);

                    let mut has_next = true;
                    if let Some(limit) = self.limit {
                        has_next = limit - scanned > 0;
                        self.limit = Some(limit - scanned);
                    }

                    if res.last_evaluated_key.is_none() || !has_next {
                        let next_token = if res.last_evaluated_key.is_some() {
                            Some(::raiden::NextToken::from_attr(&res.last_evaluated_key.unwrap()))
                        } else {
                            None
                        };
                        return Ok(::raiden::query::QueryOutput {
                            consumed_capacity: res.consumed_capacity,
                            count: res.count,
                            items,
                            next_token,
                            scanned_count: res.scanned_count,
                        })
                    }

                    self.builder = self.builder
                        .set_exclusive_start_key(res.last_evaluated_key);
                }
            }

            async fn inner_run(
                #inner_run_args
                client: ::raiden::Client,
                builder: ::raiden::aws_sdk::operation::query::builders::QueryInputBuilder,
            ) -> Result<#query_output_item, ::raiden::RaidenError> {
                let res = #api_call_token?;
                Ok(#query_output_item {
                    consumed_capacity: res.consumed_capacity,
                    count: Some(res.count as i64),
                    items: res.items,
                    last_evaluated_key: res.last_evaluated_key,
                    scanned_count: Some(res.scanned_count as i64),
                })
            }
        }

        impl<'a, T, I> #projected_builder_name<'a, T, I>
        where
            I: ::raiden::RaidenIndexItem<#struct_name>,
        {
            /// Replaces the projection item type while keeping query state.
            pub fn project<J>(self) -> #projected_builder_name<'a, T, J>
            where
                J: ::raiden::RaidenIndexItem<#struct_name>,
            {
                self.inner.project::<J>()
            }

            /// Enables strongly consistent reads for the query.
            pub fn consistent(mut self) -> Self {
                self.inner = self.inner.consistent();
                self
            }

            /// Sets the pagination token used to resume the query.
            pub fn next_token(mut self, token: ::raiden::NextToken) -> Self {
                self.inner = self.inner.next_token(token);
                self
            }

            /// Requests results in descending sort-key order.
            pub fn desc(mut self) -> Self {
                self.inner = self.inner.desc();
                self
            }

            /// Requests results in ascending sort-key order.
            pub fn asc(mut self) -> Self {
                self.inner = self.inner.asc();
                self
            }

            /// Limits the number of returned items.
            pub fn limit(mut self, limit: usize) -> Self {
                self.inner = self.inner.limit(limit);
                self
            }

            /// Applies a filter expression while preserving the projection type.
            pub fn filter(mut self, cond: impl ::raiden::filter_expression::FilterExpressionBuilder<#filter_expression_token_name>) -> Self {
                self.inner = self.inner.filter(cond);
                self
            }

            /// Applies a key condition while preserving the projection type.
            pub fn key_condition<U>(mut self, cond: impl ::raiden::key_condition::KeyConditionBuilder<T, U>) -> Self {
                self.inner = self.inner.key_condition(cond);
                self
            }

            /// Runs the query and decodes items into the projection item type.
            pub async fn run(self) -> Result<::raiden::query::QueryOutput<I>, ::raiden::RaidenError> {
                self.inner.run_inner::<I>().await
            }

            /// Runs the query with another projection item type.
            ///
            /// This remains available as a convenience wrapper when callers want
            /// to switch projection types late in the builder chain.
            pub async fn run_with<J>(self) -> Result<::raiden::query::QueryOutput<J>, ::raiden::RaidenError>
            where
                J: ::raiden::RaidenIndexItem<#struct_name>,
            {
                self.project::<J>().run().await
            }
        }
    }
}
