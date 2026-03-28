use convert_case::{Case, Casing};
use quote::*;

fn create_gsi_token_name(struct_name: &proc_macro2::Ident, index_name: &str) -> proc_macro2::Ident {
    format_ident!(
        "{}{}KeyConditionToken",
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
    let query_output_item = format_ident!("{}QueryOutput", struct_name);

    let filter_expression_token_name = format_ident!("{}FilterExpressionToken", struct_name);
    let key_condition_token_name = format_ident!("{}KeyConditionToken", struct_name);
    let gsi_tokens = gsi_definitions
        .iter()
        .filter(|gsi| gsi.partition_key.is_some())
        .map(|gsi| {
            let token_name = create_gsi_token_name(struct_name, &gsi.name);
            quote! {
                pub struct #token_name;
            }
        });
    let gsi_key_condition_methods = gsi_definitions.iter().filter_map(|gsi| {
        let partition_key = gsi.partition_key.as_ref()?;
        let method_name = format!("{}_key_condition", gsi.name.to_case(Case::Snake));
        let method_ident = if crate::helpers::is_reserved(&method_name) {
            format_ident!("r#{}", method_name)
        } else {
            format_ident!("{}", method_name)
        };
        let token_name = create_gsi_token_name(struct_name, &gsi.name);
        let attr_name = resolve_attr_name(fields, rename_all_type, partition_key);

        Some(quote! {
            pub fn #method_ident() -> ::raiden::KeyCondition<#token_name> {
                ::raiden::KeyCondition {
                    attr: #attr_name.to_owned(),
                    _token: std::marker::PhantomData,
                }
            }
        })
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
            .map(|gsi| create_gsi_token_name(struct_name, &gsi.name));

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
                        _token: std::marker::PhantomData,
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

    let from_item = super::expand_attr_to_item(format_ident!("res_item"), fields, rename_all_type);
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
                    _token: std::marker::PhantomData,
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

            pub fn key_condition(mut self, cond: impl ::raiden::key_condition::KeyConditionBuilder<T>) -> Self {
                let (cond_str, attr_names, attr_values) = cond.build();

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

            pub async fn run(mut self) -> Result<::raiden::query::QueryOutput<#struct_name>, ::raiden::RaidenError> {
                if let Some(token) = self.next_token {
                    self.builder = self.builder
                        .set_exclusive_start_key(Some(token.into_attr_values()?));
                    }

                let mut items: Vec<#struct_name> = vec![];
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
                            let mut res_item = res_item;
                            items.push(#struct_name {
                                #(#from_item)*
                            })
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
    }
}
