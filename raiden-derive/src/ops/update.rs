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
    let from_item = super::expand_attr_to_item(&format_ident!("res_item"), fields, rename_all_type);
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
                    let mut input = ::raiden::UpdateItemInput::default();
                    let pk_attr: AttributeValue = pk.into().into_attr();
                    let sk_attr: AttributeValue = sk.into().into_attr();
                    let mut key_set: std::collections::HashMap<String, AttributeValue> = std::collections::HashMap::new();
                    key_set.insert(stringify!(#partition_key_ident).to_owned(), pk_attr);
                    key_set.insert(stringify!(#sort_key_ident).to_owned(), sk_attr);
                    input.key = key_set;
                    input.table_name = self.table_name();
                    #builder_name {
                        client: &self.client,
                        input,
                        set_items: vec![],
                        add_items: vec![],
                        remove_items: vec![],
                        delete_items: vec![],
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
                    let mut input = ::raiden::UpdateItemInput::default();
                    let key_attr: AttributeValue = key.into().into_attr();
                    let mut key_set: std::collections::HashMap<String, AttributeValue> = std::collections::HashMap::new();
                    key_set.insert(stringify!(#partition_key_ident).to_owned(), key_attr);
                    input.key = key_set;
                    input.table_name = self.table_name();
                    #builder_name {
                        client: &self.client,
                        input,
                        set_items: vec![],
                        add_items: vec![],
                        remove_items: vec![],
                        delete_items: vec![],
                    }
                }
            }
        }
    };

    quote! {
        #[derive(Debug, Clone, PartialEq)]
        pub struct #item_output_name {
            // #(#output_fields)*
        }

        struct #update_expression_name;

        impl #struct_name {
            fn update_expression() -> #update_expression_name {
                #update_expression_name
            }
        }

        impl #update_expression_name {
            fn set(&self, attr: #attr_enum_name) -> ::raiden::update_expression::Set<#attr_enum_name> {
                ::raiden::update_expression::Set::new(attr)
            }

            fn add(&self, attr: #attr_enum_name) -> ::raiden::update_expression::Add<#attr_enum_name> {
                ::raiden::update_expression::Add::new(attr)
            }

            fn delete(&self, attr: #attr_enum_name) -> ::raiden::update_expression::Delete<#attr_enum_name> {
                ::raiden::update_expression::Delete::new(attr)
            }
        }

        #client_trait

        pub struct #builder_name<'a> {
            pub client: &'a ::raiden::DynamoDbClient,
            pub input: ::raiden::UpdateItemInput,
            pub add_items: Vec<(String, ::raiden::AttributeNames, ::raiden::AttributeValues)>,
            pub set_items: Vec<::raiden::update_expression::SetOrRemove>,
            pub remove_items: Vec<#attr_enum_name>,
            pub delete_items: Vec<(String, ::raiden::AttributeNames, ::raiden::AttributeValues)>,
        }

        impl<'a> #builder_name<'a> {
            pub fn raw_input(mut self, input: ::raiden::UpdateItemInput) -> Self {
                self.input = input;
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
                self.input.return_values = Some("ALL_OLD".to_owned());
                self
            }

            // INFO: raiden supports only none, all_old and all_new to map response to struct.
            pub fn return_all_new(mut self) -> Self {
                self.input.return_values = Some("ALL_NEW".to_owned());
                self
            }

            pub fn condition(mut self, cond: impl ::raiden::condition::ConditionBuilder<#condition_token_name>) -> Self {
                let (cond_str, attr_names, attr_values) = cond.build();
                if !attr_names.is_empty() {
                    self.input.expression_attribute_names = Some(attr_names);
                }
                if !attr_values.is_empty() {
                    self.input.expression_attribute_values = Some(attr_values);
                }
                self.input.condition_expression = Some(cond_str);
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
                if self.input.expression_attribute_names.is_none() {
                    if names.is_empty() {
                        self.input.expression_attribute_names = None;
                    } else {
                        self.input.expression_attribute_names = Some(names);
                    }
                } else {
                    self.input.expression_attribute_names = Some(::raiden::merge_map(self.input.expression_attribute_names.unwrap(), names));
                }

                if self.input.expression_attribute_values.is_none() {
                    if values.is_empty() {
                        self.input.expression_attribute_values = None;
                    } else {
                        self.input.expression_attribute_values = Some(values);
                    }
                } else {
                    self.input.expression_attribute_values = Some(::raiden::merge_map(self.input.expression_attribute_values.unwrap(), values));
                }

                if expression != "" {
                    self.input.update_expression = Some(expression);
                }

                let has_return_values = self.input.return_values.is_some();
                let res = self.client.update_item(self.input).await?;

                let item = if has_return_values {
                    let res_item = &res.attributes.unwrap();
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
        }
    }
}
