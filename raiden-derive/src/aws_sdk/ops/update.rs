use proc_macro2::*;
use quote::*;
use syn::*;

pub(crate) fn expand_update_item(
    partition_key: &(Ident, Type),
    sort_key: &Option<(Ident, Type)>,
    fields: &FieldsNamed,
    attr_enum_name: &Ident,
    struct_name: &Ident,
    rename_all_type: crate::rename::RenameAllType,
) -> TokenStream {
    let item_output_name = format_ident!("{}UpdateItemOutput", struct_name);
    let trait_name = format_ident!("{}UpdateItem", struct_name);
    let update_expression_name = format_ident!("{}UpdateExpression", struct_name);
    let client_name = format_ident!("{}Client", struct_name);
    let builder_name = format_ident!("{}UpdateItemBuilder", struct_name);
    let from_item = super::expand_attr_to_item(format_ident!("res_item"), fields, rename_all_type);
    let condition_token_name = format_ident!("{}ConditionToken", struct_name);
    let (partition_key_ident, partition_key_type) = partition_key;

    let client_trait = if let Some(sort_key) = sort_key {
        let (sort_key_ident, sort_key_type) = sort_key;
        quote! {
            pub trait #trait_name {
                fn update(&self, pk: impl Into<#partition_key_type>, sk: impl Into<#sort_key_type>) -> #builder_name;
            }

            impl #trait_name for #client_name {
                fn update(&self, pk: impl Into<#partition_key_type>, sk: impl Into<#sort_key_type>) -> #builder_name {
                    use std::iter::FromIterator;

                    let pk_attr: ::raiden::aws_sdk::types::AttributeValue = pk.into().into_attr();
                    let sk_attr: ::raiden::aws_sdk::types::AttributeValue = sk.into().into_attr();
                    let key_set: std::collections::HashMap<String, ::raiden::aws_sdk::types::AttributeValue> = std::collections::HashMap::from_iter([
                        (stringify!(#partition_key_ident).to_owned(), pk_attr),
                        (stringify!(#sort_key_ident).to_owned(), sk_attr),
                    ]);

                    let builder = ::raiden::aws_sdk::operation::update_item::UpdateItemInput::builder()
                        .set_key(Some(key_set))
                        .table_name(self.table_name());

                    #builder_name {
                        client: &self.client,
                        builder,
                        set_items: vec![],
                        add_items: vec![],
                        remove_items: vec![],
                        delete_items: vec![],
                        policy: self.retry_condition.strategy.policy(),
                        condition: &self.retry_condition,
                    }
                }
            }
        }
    } else {
        quote! {
            pub trait #trait_name {
                fn update(&self, key: impl Into<#partition_key_type>) -> #builder_name;
            }

            impl #trait_name for #client_name {
                fn update(&self, key: impl Into<#partition_key_type>) -> #builder_name {
                    use std::iter::FromIterator;

                    let key_attr: ::raiden::aws_sdk::types::AttributeValue = key.into().into_attr();
                    let key_set: std::collections::HashMap<String, ::raiden::aws_sdk::types::AttributeValue> = std::collections::HashMap::from_iter([
                        (stringify!(#partition_key_ident).to_owned(), key_attr),
                    ]);

                    let builder = ::raiden::aws_sdk::operation::update_item::UpdateItemInput::builder()
                        .set_key(Some(key_set))
                        .table_name(self.table_name());

                    #builder_name {
                        client: &self.client,
                        builder,
                        set_items: vec![],
                        add_items: vec![],
                        remove_items: vec![],
                        delete_items: vec![],
                        policy: self.retry_condition.strategy.policy(),
                        condition: &self.retry_condition,
                    }
                }
            }
        }
    };

    let api_call_token = super::api_call_token!("update_item");
    let (call_inner_run, inner_run_args) = if cfg!(feature = "tracing") {
        (
            quote! {
                let table_name = builder.get_table_name().clone().expect("table name should be set");
                #builder_name::inner_run(table_name, client, builder).await
            },
            quote! { table_name: String, },
        )
    } else {
        (
            quote! { #builder_name::inner_run(client, builder).await },
            quote! {},
        )
    };

    quote! {
        #[derive(Debug, Clone, PartialEq)]
        pub struct #item_output_name {
            // #(#output_fields)*
        }

        pub struct #update_expression_name;

        impl #struct_name {
            pub fn update_expression() -> #update_expression_name {
                #update_expression_name
            }
        }

        impl #update_expression_name {
            pub fn set(&self, attr: #attr_enum_name) -> ::raiden::update_expression::Set<#attr_enum_name> {
                ::raiden::update_expression::Set::new(attr)
            }

            pub fn add(&self, attr: #attr_enum_name) -> ::raiden::update_expression::Add<#attr_enum_name> {
                ::raiden::update_expression::Add::new(attr)
            }

            pub fn delete(&self, attr: #attr_enum_name) -> ::raiden::update_expression::Delete<#attr_enum_name> {
                ::raiden::update_expression::Delete::new(attr)
            }
        }

        #client_trait

        pub struct #builder_name<'a> {
            pub client: &'a ::raiden::Client,
            pub builder: ::raiden::aws_sdk::operation::update_item::builders::UpdateItemInputBuilder,
            pub add_items: Vec<(String, ::raiden::AttributeNames, ::raiden::AttributeValues)>,
            pub set_items: Vec<::raiden::update_expression::SetOrRemove>,
            pub remove_items: Vec<#attr_enum_name>,
            pub delete_items: Vec<(String, ::raiden::AttributeNames, ::raiden::AttributeValues)>,
            pub policy: ::raiden::Policy,
            pub condition: &'a ::raiden::retry::RetryCondition,
        }

        impl<'a> #builder_name<'a> {
            pub fn raw_input(mut self, builder: ::raiden::aws_sdk::operation::update_item::builders::UpdateItemInputBuilder) -> Self {
                self.builder = builder;
                self
            }

            pub fn add(mut self, add: impl ::raiden::update_expression::UpdateAddExpressionBuilder) -> Self {
                self.add_items.push(add.build());
                self
            }

            pub fn set(mut self, set: impl ::raiden::update_expression::UpdateSetExpressionBuilder) -> Self {
                self.set_items.push(set.build());
                self
            }

            pub fn remove(mut self, attr: #attr_enum_name) -> Self {
                self.remove_items.push(attr);
                self
            }

            pub fn delete(mut self, set: impl ::raiden::update_expression::UpdateDeleteExpressionBuilder) -> Self {
                self.delete_items.push(set.build());
                self
            }

            // INFO: raiden supports only none, all_old and all_new to map response to struct.
            pub fn return_all_old(mut self) -> Self {
                self.builder = self.builder.return_values(::raiden::aws_sdk::types::ReturnValue::AllOld);
                self
            }

            // INFO: raiden supports only none, all_old and all_new to map response to struct.
            pub fn return_all_new(mut self) -> Self {
                self.builder = self.builder.return_values(::raiden::aws_sdk::types::ReturnValue::AllNew);
                self
            }

            pub fn condition(mut self, cond: impl ::raiden::condition::ConditionBuilder<#condition_token_name>) -> Self {
                let (cond_str, attr_names, attr_values) = cond.build();

                if !attr_names.is_empty() {
                    self.builder = self.builder
                        .set_expression_attribute_names(Some(attr_names));
                }

                if !attr_values.is_empty() {
                    self.builder = self.builder
                        .set_expression_attribute_values(Some(attr_values));
                }

                self.builder = self.builder.condition_expression(cond_str);
                self
            }

            fn build_expression(&mut self) -> (String, ::raiden::AttributeNames , ::raiden::AttributeValues) {
                let mut attr_names: ::raiden::AttributeNames = std::collections::HashMap::new();
                let mut attr_values: ::raiden::AttributeValues = std::collections::HashMap::new();

                let add_items = std::mem::replace(&mut self.add_items, vec![]);
                let set_items = std::mem::replace(&mut self.set_items, vec![]);
                let remove_items = std::mem::replace(&mut self.remove_items, vec![]);
                let delete_items = std::mem::replace(&mut self.delete_items, vec![]);

                let mut remove_expressions = remove_items.into_iter().map(|name| {
                    let placeholder = format!(":value{}", ::raiden::generate_value_id());
                    let attr_name = format!("#{}", name.into_attr_name());
                    let val = format!("{}", attr_name);
                    attr_names.insert(attr_name, name.into_attr_name());
                    val
                }).collect::<Vec<String>>();

                let mut set_expressions = vec![];
                for set_item in set_items {
                    match set_item {
                        raiden::update_expression::SetOrRemove::Set(expression, names, values) => {
                            attr_names = ::raiden::merge_map(attr_names, names);
                            attr_values = ::raiden::merge_map(attr_values, values);
                            set_expressions.push(expression);
                        }
                        // https://github.com/raiden-rs/raiden-dynamo/issues/64
                        // If empty set detected, replace it to remove expression.
                        raiden::update_expression::SetOrRemove::Remove(expression, names) => {
                            attr_names = ::raiden::merge_map(attr_names, names);
                            remove_expressions.push(expression);
                        }
                    }
                }
                let set_expression = set_expressions.join(", ");

                let mut add_expressions = vec![];
                for add_item in add_items {
                    let (expression, names, values) = add_item;
                    if expression != "" {
                        attr_names = ::raiden::merge_map(attr_names, names);
                        attr_values = ::raiden::merge_map(attr_values, values);
                        add_expressions.push(expression);
                    }
                }
                let add_expression = add_expressions.join(", ");

                let remove_expression = remove_expressions.join(", ");

                let mut delete_expressions = vec![];
                for delete_item in delete_items {
                    let (expression, names, values) = delete_item;
                    if expression != "" {
                        attr_names = ::raiden::merge_map(attr_names, names);
                        attr_values = ::raiden::merge_map(attr_values, values);
                        delete_expressions.push(expression);
                    }
                }
                let delete_expression = delete_expressions.join(", ");

                let mut update_expressions: Vec<String> = vec![];
                if !add_expression.is_empty() {
                    update_expressions.push(format!("ADD {}", add_expression));
                }
                if !set_expression.is_empty() {
                    update_expressions.push(format!("SET {}", set_expression));
                }
                if !remove_expression.is_empty() {
                    update_expressions.push(format!("REMOVE {}", remove_expression));
                }
                if !delete_expression.is_empty() {
                    update_expressions.push(format!("DELETE {}", delete_expression));
                }
                let update_expression = update_expressions.join(" ");
                (update_expression, attr_names, attr_values)
            }

            pub async fn run(mut self) -> Result<::raiden::update::UpdateOutput<#struct_name>, ::raiden::RaidenError> {
                let (expression, names, values) = self.build_expression();

                if self.builder.get_expression_attribute_names().is_none() {
                    if names.is_empty() {
                        self.builder = self.builder
                            .set_expression_attribute_names(None);
                    } else {
                        self.builder = self.builder
                            .set_expression_attribute_names(Some(names));
                    }
                } else {
                    let v = self
                        .builder
                        .get_expression_attribute_names()
                        .clone()
                        .unwrap();
                    self.builder = self.builder
                        .set_expression_attribute_names(Some(::raiden::merge_map(v, names)));
                    }

                if self.builder.get_expression_attribute_values().is_none() {
                    if values.is_empty() {
                        self.builder = self.builder
                            .set_expression_attribute_values(None);
                    } else {
                        self.builder = self.builder
                            .set_expression_attribute_values(Some(values));
                    }
                } else {
                    let v = self
                        .builder
                        .get_expression_attribute_values()
                        .clone()
                        .unwrap();
                    self.builder = self.builder
                        .set_expression_attribute_values(Some(::raiden::merge_map(v, values)));
                }

                if expression != "" {
                    self.builder = self.builder.update_expression(expression);
                }

                let has_return_values = self.builder.get_return_values().is_some();
                let builder = self.builder.clone();
                let client = self.client.clone();
                let policy: ::raiden::RetryPolicy = self.policy.into();

                let res = policy.retry_if(move || {
                    let builder = builder.clone();
                    let client = client.clone();
                    async { #call_inner_run }
                }, self.condition).await?;


                let item = if has_return_values {
                    let mut res_item = res.attributes.unwrap();
                    Some(#struct_name {
                        #(#from_item)*
                    })
                } else {
                    None
                };

                Ok(::raiden::update::UpdateOutput {
                    item,
                    consumed_capacity: res.consumed_capacity,
                    item_collection_metrics: res.item_collection_metrics,
                })
            }

            async fn inner_run(
                #inner_run_args
                client: ::raiden::Client,
                builder: ::raiden::aws_sdk::operation::update_item::builders::UpdateItemInputBuilder,
            ) -> Result<::raiden::aws_sdk::operation::update_item::UpdateItemOutput, ::raiden::RaidenError> {
                Ok(#api_call_token?)
            }
        }

    }
}
