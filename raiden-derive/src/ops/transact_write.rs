use crate::rename::*;
use quote::*;

pub(crate) fn expand_transact_write(
    struct_name: &proc_macro2::Ident,
    partition_key: &proc_macro2::Ident,
    _sort_key: &Option<proc_macro2::Ident>,
    fields: &syn::FieldsNamed,
    attr_enum_name: &proc_macro2::Ident,
    rename_all_type: crate::rename::RenameAllType,
    table_name: &str,
) -> proc_macro2::TokenStream {
    let item_input_name = format_ident!("{}PutItemInput", struct_name);
    // let item_output_name = format_ident!("{}PutItemOutput", struct_name);
    let put_builder = format_ident!("{}TransactPutItemBuilder", struct_name);
    let update_builder = format_ident!("{}TransactUpdateItemBuilder", struct_name);
    let delete_builder = format_ident!("{}TransactDeleteItemBuilder", struct_name);
    let condition_check_builder = format_ident!("{}TransactConditionCheckBuilder", struct_name);
    let condition_token_name = format_ident!("{}ConditionToken", struct_name);

    // let output_values = fields.named.iter().map(|f| {
    //     let ident = &f.ident.clone().unwrap();
    //     let renamed = crate::finder::find_rename_value(&f.attrs);
    //     let attr_key = create_renamed(ident.to_string(), renamed, rename_all_type);
    //     if crate::finder::include_unary_attr(&f.attrs, "uuid") {
    //         quote! {
    //             #ident: uuid_map.get(#attr_key).cloned().unwrap().into(),
    //         }
    //     } else {
    //         quote! {
    //             #ident: item.#ident,
    //         }
    //     }
    // });

    let input_items = {
        let insertion = fields.named.iter().map(|f| {
            let ident = &f.ident.clone().unwrap();
            let renamed = crate::finder::find_rename_value(&f.attrs);
            let attr_key = create_renamed(ident.to_string(), renamed, rename_all_type);
            if crate::finder::include_unary_attr(&f.attrs, "uuid") {
                quote! {
                    let id = #struct_name::gen();
                    input_item.insert(
                        #attr_key.to_string(),
                        id.clone().into_attr(),
                    );
                    uuid_map.insert(
                        #attr_key.to_string(),
                        id,
                    );
                }
            } else {
                quote! {
                    let value = item.#ident.into_attr();
                    if !::raiden::is_attr_value_empty(&value) {
                        input_item.insert(
                            #attr_key.to_string(),
                            value,
                        );
                    }
                }
            }
        });

        quote! {
            let mut input_item: std::collections::HashMap<String, raiden::AttributeValue> = std::collections::HashMap::new();
            #(#insertion)*
        }
    };

    quote! {
        impl #struct_name {
            pub fn put(item: #item_input_name) -> #put_builder {
                let mut input = ::raiden::Put::default();
                let mut attribute_names: std::collections::HashMap<String, String> = std::collections::HashMap::new();
                let mut attribute_values: std::collections::HashMap<String, raiden::AttributeValue> = std::collections::HashMap::new();
                let mut uuid_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();

                #input_items

                // let output_item = #item_output_name {
                //     #(#output_values)*
                // };
                input.item = input_item;
                #put_builder {
                    input,
                    table_name: #table_name.to_owned(),
                    table_prefix: "".to_owned(),
                    table_suffix: "".to_owned(),
                    // item: output_item,
                }
            }

            // TODO: Support sort key
            pub fn condition_check<K>(key: K) -> #condition_check_builder where K: ::raiden::IntoAttribute + std::marker::Send {
                let mut input = ::raiden::ConditionCheck::default();
                let key_attr: AttributeValue = key.into_attr();
                let mut key_set: std::collections::HashMap<String, AttributeValue> = std::collections::HashMap::new();
                key_set.insert(stringify!(#partition_key).to_owned(), key_attr);
                input.key = key_set;
                #condition_check_builder {
                    input,
                    table_name: #table_name.to_owned(),
                    table_prefix: "".to_owned(),
                    table_suffix: "".to_owned(),
                    // item: output_item,
                }
            }

            // TODO: Support sort key
            pub fn delete<K>(key: K) -> #delete_builder where K: ::raiden::IntoAttribute + std::marker::Send {
                let mut input = ::raiden::Delete::default();
                let key_attr: AttributeValue = key.into_attr();
                let mut key_set: std::collections::HashMap<String, AttributeValue> = std::collections::HashMap::new();
                key_set.insert(stringify!(#partition_key).to_owned(), key_attr);
                input.key = key_set;
                #delete_builder {
                    input,
                    table_name: #table_name.to_owned(),
                    table_prefix: "".to_owned(),
                    table_suffix: "".to_owned(),
                    // item: output_item,
                }
            }

            // TODO: Support sort key
            pub fn update<K>(key: K) -> #update_builder where K: ::raiden::IntoAttribute + std::marker::Send {
                let mut input = ::raiden::Update::default();

                let key_attr: AttributeValue = key.into_attr();
                let mut key_set: std::collections::HashMap<String, AttributeValue> = std::collections::HashMap::new();
                key_set.insert(stringify!(#partition_key).to_owned(), key_attr);
                input.key = key_set;

                #update_builder {
                    input,
                    table_name: #table_name.to_owned(),
                    table_prefix: "".to_owned(),
                    table_suffix: "".to_owned(),
                    // item: output_item,
                    add_items: vec![],
                    set_items: vec![],
                    remove_items: vec![],
                    delete_items: vec![],
                }
            }
        }

        pub struct #put_builder {
            pub table_name: String,
            pub table_prefix: String,
            pub table_suffix: String,
            pub input: ::raiden::Put,
        }

        impl ::raiden::TransactWritePutBuilder for #put_builder {
            fn build(self) -> ::raiden::Put {
                let mut input = self.input;
                input.table_name = format!("{}{}{}", self.table_prefix, self.table_name, self.table_suffix);
                input
            }
        }

        impl #put_builder {

            fn table_prefix(mut self, s: impl Into<String>) -> Self {
                self.table_prefix = s.into();
                self
            }

            fn table_suffix(mut self, s: impl Into<String>) -> Self {
                self.table_suffix = s.into();
                self
            }

            fn condition(mut self, cond: impl ::raiden::condition::ConditionBuilder<#condition_token_name>) -> Self {
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
        }

        pub struct #update_builder {
            pub table_name: String,
            pub table_prefix: String,
            pub table_suffix: String,
            pub input: ::raiden::Update,

            pub add_items: Vec<(String, ::raiden::AttributeNames, ::raiden::AttributeValues)>,
            pub set_items: Vec<::raiden::update_expression::SetOrRemove>,
            pub remove_items: Vec<#attr_enum_name>,
            pub delete_items: Vec<(#attr_enum_name, ::raiden::AttributeValue)>,
        }

        impl ::raiden::TransactWriteUpdateBuilder for #update_builder {
            fn build(mut self) -> ::raiden::Update {
                // let mut input = self.input;

                // TODO: Refactor later
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

                let remove_expression = remove_expressions.join(", ");

                let mut add_expressions = vec![];
                for add_item in add_items {
                    let (expression, names, values) = add_item;
                    attr_names = ::raiden::merge_map(attr_names, names);
                    attr_values = ::raiden::merge_map(attr_values, values);
                    add_expressions.push(expression);
                }
                let add_expression = add_expressions.join(", ");



                let delete_expression = delete_items.into_iter().map(|(name, value)| {
                    let placeholder = format!(":value{}", ::raiden::generate_value_id());
                    let attr_name = format!("#{}", name.into_attr_name());
                    let val = format!("{} {}", attr_name, placeholder);
                    attr_names.insert(attr_name, name.into_attr_name());
                    attr_values.insert(placeholder, value);
                    val
                }).collect::<Vec<_>>().join(", ");

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

                if self.input.expression_attribute_names.is_none() {
                    self.input.expression_attribute_names = Some(attr_names);
                } else {
                    self.input.expression_attribute_names = Some(::raiden::merge_map(self.input.expression_attribute_names.unwrap(), attr_names));
                }
                if self.input.expression_attribute_values.is_none() {
                    self.input.expression_attribute_values = Some(attr_values);
                } else {
                    self.input.expression_attribute_values = Some(::raiden::merge_map(self.input.expression_attribute_values.unwrap(), attr_values));
                }

                self.input.update_expression = update_expression;

                self.input.table_name = format!("{}{}{}", self.table_prefix, self.table_name, self.table_suffix);
                self.input
            }
        }

        impl #update_builder {

            fn table_prefix(mut self, s: impl Into<String>) -> Self {
                self.table_prefix = s.into();
                self
            }

            fn table_suffix(mut self, s: impl Into<String>) -> Self {
                self.table_suffix = s.into();
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

            pub fn delete(mut self, attr: #attr_enum_name, value: impl ::raiden::IntoAttribute) -> Self {
                self.delete_items.push((attr, value.into_attr()));
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
        }

        pub struct #delete_builder {
            pub table_name: String,
            pub table_prefix: String,
            pub table_suffix: String,
            pub input: ::raiden::Delete,
        }

        impl ::raiden::TransactWriteDeleteBuilder for #delete_builder {
            fn build(self) -> ::raiden::Delete {
                let mut input = self.input;
                input.table_name = format!("{}{}{}", self.table_prefix, self.table_name, self.table_suffix);
                input
            }
        }

        impl #delete_builder {
            fn table_prefix(mut self, s: impl Into<String>) -> Self {
                self.table_prefix = s.into();
                self
            }

            fn table_suffix(mut self, s: impl Into<String>) -> Self {
                self.table_suffix = s.into();
                self
            }

            fn condition(mut self, cond: impl ::raiden::condition::ConditionBuilder<#condition_token_name>) -> Self {
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
        }

        pub struct #condition_check_builder {
            pub table_name: String,
            pub table_prefix: String,
            pub table_suffix: String,
            pub input: ::raiden::ConditionCheck,
        }

        impl ::raiden::TransactWriteConditionCheckBuilder for #condition_check_builder {
            fn build(self) -> ::raiden::ConditionCheck {
                let mut input = self.input;
                input.table_name = format!("{}{}{}", self.table_prefix, self.table_name, self.table_suffix);
                input
            }
        }

        impl #condition_check_builder {
            fn table_prefix(mut self, s: impl Into<String>) -> Self {
                self.table_prefix = s.into();
                self
            }

            fn table_suffix(mut self, s: impl Into<String>) -> Self {
                self.table_suffix = s.into();
                self
            }

            fn condition(mut self, cond: impl ::raiden::condition::ConditionBuilder<#condition_token_name>) -> Self {
                let (cond_str, attr_names, attr_values) = cond.build();
                if !attr_names.is_empty() {
                    self.input.expression_attribute_names = Some(attr_names);
                }
                if !attr_values.is_empty() {
                    self.input.expression_attribute_values = Some(attr_values);
                }
                self.input.condition_expression = cond_str;
                self
            }
        }

    }
}

/*
#[derive(Default, Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "deserialize_structs", derive(Deserialize))]
pub struct Put {
    /// <p>A condition that must be satisfied in order for a conditional update to succeed.</p>
    #[serde(rename = "ConditionExpression")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition_expression: Option<String>,
    /// <p>One or more substitution tokens for attribute names in an expression.</p>
    #[serde(rename = "ExpressionAttributeNames")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expression_attribute_names: Option<::std::collections::HashMap<String, String>>,
    /// <p>One or more values that can be substituted in an expression.</p>
    #[serde(rename = "ExpressionAttributeValues")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expression_attribute_values: Option<::std::collections::HashMap<String, AttributeValue>>,
    /// <p>A map of attribute name to attribute values, representing the primary key of the item to be written by <code>PutItem</code>. All of the table's primary key attributes must be specified, and their data types must match those of the table's key schema. If any attributes are present in the item that are part of an index key schema for the table, their types must match the index key schema. </p>
    #[serde(rename = "Item")]
    pub item: ::std::collections::HashMap<String, AttributeValue>,
    /// <p>Use <code>ReturnValuesOnConditionCheckFailure</code> to get the item attributes if the <code>Put</code> condition fails. For <code>ReturnValuesOnConditionCheckFailure</code>, the valid values are: NONE and ALL_OLD.</p>
    #[serde(rename = "ReturnValuesOnConditionCheckFailure")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_values_on_condition_check_failure: Option<String>,
    /// <p>Name of the table in which to write the item.</p>
    #[serde(rename = "TableName")]
    pub table_name: String,
}*/
