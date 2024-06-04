use proc_macro2::*;
use quote::*;
use syn::*;

use crate::rename::*;

pub(crate) fn expand_batch_get(
    partition_key: &(Ident, Type),
    sort_key: &Option<(Ident, Type)>,
    struct_name: &Ident,
    fields: &FieldsNamed,
    rename_all_type: RenameAllType,
) -> proc_macro2::TokenStream {
    let trait_name = format_ident!("{}BatchGetItem", struct_name);
    let client_name = format_ident!("{}Client", struct_name);
    let builder_name = format_ident!("{}BatchGetItemBuilder", struct_name);
    let from_item = super::expand_attr_to_item(format_ident!("res_item"), fields, rename_all_type);
    let (partition_key_ident, partition_key_type) = partition_key;

    let builder_keys_type = if sort_key.is_none() {
        quote! { std::vec::Vec<::raiden::AttributeValue> }
    } else {
        quote! { std::vec::Vec<(::raiden::AttributeValue, ::raiden::AttributeValue)> }
    };

    let insertion_attribute_name = fields.named.iter().map(|f| {
        let ident = &f.ident.clone().unwrap();
        let renamed = crate::finder::find_rename_value(&f.attrs);
        let result = create_renamed(ident.to_string(), renamed, rename_all_type);
        quote! {
            names.insert(
                format!("#{}", #result.clone()),
                #result.to_string(),
            );
        }
    });

    let builder_init = quote! {
        let names = {
            let mut names: ::raiden::AttributeNames = std::collections::HashMap::new();
            #(#insertion_attribute_name)*
            names
        };
        let projection_expression = Some(names.keys().map(|v| v.to_string()).collect::<Vec<String>>().join(", "));

        #builder_name {
            client: &self.client,
            table_name: self.table_name(),
            keys: key_attrs,
            attribute_names: Some(names),
            projection_expression
        }
    };

    let client_trait = if sort_key.is_none() {
        quote! {
            pub trait #trait_name {
                fn batch_get(&self, keys: std::vec::Vec<impl Into<#partition_key_type>>) -> #builder_name;
            }

            impl #trait_name for #client_name {
                fn batch_get(&self, keys: std::vec::Vec<impl Into<#partition_key_type>>) -> #builder_name {
                    let mut key_attrs = vec![];
                    for key in keys.into_iter() {
                        key_attrs.push(key.into().into_attr());
                    }

                    #builder_init
                }
            }
        }
    } else {
        let (_, sort_key_type) = sort_key.clone().unwrap();
        quote! {
            pub trait #trait_name {
                fn batch_get(&self, keys: std::vec::Vec<(impl Into<#partition_key_type>, impl Into<#sort_key_type>)>) -> #builder_name;
            }

            impl #trait_name for #client_name {
                fn batch_get(&self, keys: std::vec::Vec<(impl Into<#partition_key_type>, impl Into<#sort_key_type>)>) -> #builder_name {
                    let mut key_attrs = vec![];
                    for (pk, sk) in keys.into_iter() {
                        key_attrs.push((pk.into().into_attr(), sk.into().into_attr()));
                    }

                    #builder_init
                }
            }
        }
    };

    let convert_to_external_proc = if let Some(sort_key) = sort_key {
        let (sort_key_ident, _sort_key_type) = sort_key;
        quote! {
            for (pk_attr, sk_attr) in keys.into_iter() {
                let mut key_val: std::collections::HashMap<String, ::raiden::AttributeValue> = Default::default();
                key_val.insert(stringify!(#partition_key_ident).to_owned(), pk_attr);
                key_val.insert(stringify!(#sort_key_ident).to_owned(), sk_attr);
                item.keys.push(key_val);
            }
        }
    } else {
        quote! {
            for key_attr in keys.into_iter() {
                let mut key_val: std::collections::HashMap<String, ::raiden::AttributeValue> = Default::default();
                key_val.insert(stringify!(#partition_key_ident).to_owned(), key_attr);
                item.keys.push(key_val);
            }
        }
    };

    let api_call_token = super::api_call_token!("batch_get_item");
    let (call_inner_run, inner_run_args) = if cfg!(feature = "tracing") {
        (
            quote! { #builder_name::inner_run(&self.table_name, &self.client, input).await? },
            quote! { table_name: &str, },
        )
    } else {
        (
            quote! { #builder_name::inner_run(&self.client, input).await? },
            quote! {},
        )
    };

    quote! {
        #client_trait

        pub struct #builder_name<'a> {
            pub client: &'a ::raiden::DynamoDbClient,
            pub table_name: String,
            pub keys: #builder_keys_type,
            pub attribute_names: Option<::raiden::AttributeNames>,
            pub projection_expression: Option<String>
        }

        impl<'a> #builder_name<'a> {

            #![allow(clippy::field_reassign_with_default)]
            pub async fn run(mut self) -> Result<::raiden::batch_get::BatchGetOutput<#struct_name>, ::raiden::RaidenError> {
                let mut items: std::vec::Vec<#struct_name> = vec![];
                let mut unprocessed_keys = ::raiden::KeysAndAttributes::default();

                // TODO: for now set 5, however we should make it more flexible.
                let mut unprocessed_retry = 5;
                loop {
                    let mut input = ::raiden::BatchGetItemInput::default();
                    let mut item = ::raiden::KeysAndAttributes::default();
                    item.keys = Default::default();
                    item.expression_attribute_names = self.attribute_names.clone();
                    item.projection_expression = self.projection_expression.clone();

                    for key in unprocessed_keys.keys.iter() {
                        item.keys.push(key.clone());
                    }

                    if unprocessed_keys.keys.len() < 100 {
                        let keys = self.keys.drain(0..std::cmp::min(100 - unprocessed_keys.keys.len(), self.keys.len()));
                        #convert_to_external_proc
                    }

                    input.request_items = Default::default();
                    input
                        .request_items
                        .insert(self.table_name.to_string(), item);

                    let res = #call_inner_run;

                    if self.keys.is_empty() {
                        unprocessed_retry -= 1;
                    }

                    if let Some(res_responses) = res.responses {
                        let mut res_responses = res_responses;
                        if let Some(res_items) = (&mut res_responses).remove(&self.table_name) {
                            for res_item in res_items.into_iter() {
                                let mut res_item = res_item;
                                items.push(#struct_name {
                                    #(#from_item)*
                                })
                            }
                        } else {
                            return Err(::raiden::RaidenError::ResourceNotFound(format!("'{}' table not found or not active", &self.table_name)));
                        }
                    } else {
                        return Err(::raiden::RaidenError::ResourceNotFound("resource not found".to_owned()));
                    }

                    unprocessed_keys.keys = vec![];

                    if let Some(mut keys_by_table) = res.unprocessed_keys {
                        if let Some(mut keys_attrs) = keys_by_table.get_mut(&self.table_name) {
                            unprocessed_keys.keys = keys_attrs.keys.clone();
                        }
                    }


                    if (self.keys.is_empty() && unprocessed_keys.keys.is_empty()) || unprocessed_retry == 0 {
                        return Ok(::raiden::batch_get::BatchGetOutput {
                            consumed_capacity: res.consumed_capacity,
                            items,
                            unprocessed_keys: Some(unprocessed_keys),
                        })
                    }
                }
            }

            async fn inner_run(
                #inner_run_args
                client: &::raiden::DynamoDbClient,
                input: ::raiden::BatchGetItemInput,
            ) -> Result<::raiden::BatchGetItemOutput, ::raiden::RaidenError> {
                Ok(#api_call_token?)
            }
        }
    }
}

/*
https://docs.aws.amazon.com/ja_jp/sdk-for-javascript/v2/developer-guide/dynamodb-example-table-read-write-batch.html

pub struct BatchGetItemInput {
    /// <p><p>A map of one or more table names and, for each table, a map that describes one or more items to retrieve from that table. Each table name can be used only once per <code>BatchGetItem</code> request.</p> <p>Each element in the map of items to retrieve consists of the following:</p> <ul> <li> <p> <code>ConsistentRead</code> - If <code>true</code>, a strongly consistent read is used; if <code>false</code> (the default), an eventually consistent read is used.</p> </li> <li> <p> <code>ExpressionAttributeNames</code> - One or more substitution tokens for attribute names in the <code>ProjectionExpression</code> parameter. The following are some use cases for using <code>ExpressionAttributeNames</code>:</p> <ul> <li> <p>To access an attribute whose name conflicts with a DynamoDB reserved word.</p> </li> <li> <p>To create a placeholder for repeating occurrences of an attribute name in an expression.</p> </li> <li> <p>To prevent special characters in an attribute name from being misinterpreted in an expression.</p> </li> </ul> <p>Use the <b>#</b> character in an expression to dereference an attribute name. For example, consider the following attribute name:</p> <ul> <li> <p> <code>Percentile</code> </p> </li> </ul> <p>The name of this attribute conflicts with a reserved word, so it cannot be used directly in an expression. (For the complete list of reserved words, see <a href="https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/ReservedWords.html">Reserved Words</a> in the <i>Amazon DynamoDB Developer Guide</i>). To work around this, you could specify the following for <code>ExpressionAttributeNames</code>:</p> <ul> <li> <p> <code>{&quot;#P&quot;:&quot;Percentile&quot;}</code> </p> </li> </ul> <p>You could then use this substitution in an expression, as in this example:</p> <ul> <li> <p> <code>#P = :val</code> </p> </li> </ul> <note> <p>Tokens that begin with the <b>:</b> character are <i>expression attribute values</i>, which are placeholders for the actual value at runtime.</p> </note> <p>For more information about expression attribute names, see <a href="https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Expressions.AccessingItemAttributes.html">Accessing Item Attributes</a> in the <i>Amazon DynamoDB Developer Guide</i>.</p> </li> <li> <p> <code>Keys</code> - An array of primary key attribute values that define specific items in the table. For each primary key, you must provide <i>all</i> of the key attributes. For example, with a simple primary key, you only need to provide the partition key value. For a composite key, you must provide <i>both</i> the partition key value and the sort key value.</p> </li> <li> <p> <code>ProjectionExpression</code> - A string that identifies one or more attributes to retrieve from the table. These attributes can include scalars, sets, or elements of a JSON document. The attributes in the expression must be separated by commas.</p> <p>If no attribute names are specified, then all attributes are returned. If any of the requested attributes are not found, they do not appear in the result.</p> <p>For more information, see <a href="https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Expressions.AccessingItemAttributes.html">Accessing Item Attributes</a> in the <i>Amazon DynamoDB Developer Guide</i>.</p> </li> <li> <p> <code>AttributesToGet</code> - This is a legacy parameter. Use <code>ProjectionExpression</code> instead. For more information, see <a href="https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/LegacyConditionalParameters.AttributesToGet.html">AttributesToGet</a> in the <i>Amazon DynamoDB Developer Guide</i>. </p> </li> </ul></p>
    #[serde(rename = "RequestItems")]
    pub request_items: ::std::collections::HashMap<String, KeysAndAttributes>,
    #[serde(rename = "ReturnConsumedCapacity")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_consumed_capacity: Option<String>,
}

pub struct KeysAndAttributes {
    pub consistent_read: Option<bool>,
    pub expression_attribute_names: Option<::std::collections::HashMap<String, String>>,
    pub keys: Vec<::std::collections::HashMap<String, AttributeValue>>,
    pub projection_expression: Option<String>,
}

pub struct BatchGetItemOutput {
    /// <p><p>The read capacity units consumed by the entire <code>BatchGetItem</code> operation.</p> <p>Each element consists of:</p> <ul> <li> <p> <code>TableName</code> - The table that consumed the provisioned throughput.</p> </li> <li> <p> <code>CapacityUnits</code> - The total number of capacity units consumed.</p> </li> </ul></p>
    #[serde(rename = "ConsumedCapacity")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consumed_capacity: Option<Vec<ConsumedCapacity>>,
    /// <p>A map of table name to a list of items. Each object in <code>Responses</code> consists of a table name, along with a map of attribute data consisting of the data type and attribute value.</p>
    #[serde(rename = "Responses")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub responses: Option<
        ::std::collections::HashMap<
            String,
            Vec<::std::collections::HashMap<String, AttributeValue>>,
        >,
    >,
    /// <p>A map of tables and their respective keys that were not processed with the current response. The <code>UnprocessedKeys</code> value is in the same form as <code>RequestItems</code>, so the value can be provided directly to a subsequent <code>BatchGetItem</code> operation. For more information, see <code>RequestItems</code> in the Request Parameters section.</p> <p>Each element consists of:</p> <ul> <li> <p> <code>Keys</code> - An array of primary key attribute values that define specific items in the table.</p> </li> <li> <p> <code>ProjectionExpression</code> - One or more attributes to be retrieved from the table or index. By default, all attributes are returned. If a requested attribute is not found, it does not appear in the result.</p> </li> <li> <p> <code>ConsistentRead</code> - The consistency of a read operation. If set to <code>true</code>, then a strongly consistent read is used; otherwise, an eventually consistent read is used.</p> </li> </ul> <p>If there are no unprocessed keys remaining, the response contains an empty <code>UnprocessedKeys</code> map.</p>
    #[serde(rename = "UnprocessedKeys")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unprocessed_keys: Option<::std::collections::HashMap<String, KeysAndAttributes>>,
}

*/
