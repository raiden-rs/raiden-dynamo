use crate::rename::*;
use quote::*;

pub(crate) fn expand_put_item(
    partition_key: &proc_macro2::Ident,
    struct_name: &proc_macro2::Ident,
    fields: &syn::FieldsNamed,
    rename_all_type: crate::rename::RenameAllType,
) -> proc_macro2::TokenStream {
    let item_input_name = format_ident!("{}PutItemInput", struct_name);
    let item_input_builder_name = format_ident!("{}PutItemInputBuilder", struct_name);
    let item_output_name = format_ident!("{}PutItemOutput", struct_name);
    let trait_name = format_ident!("{}PutItem", struct_name);
    let client_name = format_ident!("{}Client", struct_name);
    let builder_name = format_ident!("{}PutItemBuilder", struct_name);
    let condition_token_name = format_ident!("{}ConditionToken", struct_name);

    let input_fields = fields
        .named
        .iter()
        .filter(|f| !crate::finder::include_unary_attr(&f.attrs, "uuid"))
        .map(|f| {
            let ident = &f.ident.clone().unwrap();
            let ty = &f.ty;
            quote! {
                #ident: #ty,
            }
        });

    let output_fields = fields.named.iter().map(|f| {
        let ident = &f.ident.clone().unwrap();
        let ty = &f.ty;
        quote! {
            pub #ident: #ty,
        }
    });

    let output_values = fields.named.iter().map(|f| {
        let ident = &f.ident.clone().unwrap();
        let renamed = crate::finder::find_rename_value(&f.attrs);
        let attr_key = create_renamed(ident.to_string(), renamed, rename_all_type);
        if crate::finder::include_unary_attr(&f.attrs, "uuid") {
            quote! {
                #ident: uuid_map.get(#attr_key).cloned().unwrap().into(),
            }
        } else {
            quote! {
                #ident: item.#ident,
            }
        }
    });

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
                    input_item.insert(
                        #attr_key.to_string(),
                        item.#ident.clone().into_attr(),
                    );
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
            pub fn put_item_builder() -> #item_input_builder_name {
                #item_input_builder_name::default()
            }
        }

        #[derive(Debug, Clone, PartialEq, Builder)]
        #[builder(setter(into))]
        pub struct #item_input_name {
            #(#input_fields)*
        }

        #[derive(Debug, Clone, PartialEq)]
        pub struct #item_output_name {
            #(#output_fields)*
        }

        pub trait #trait_name {
            fn put(&self, item: #item_input_name) -> #builder_name;
        }

        impl #trait_name for #client_name {
            fn put(&self, item: #item_input_name) -> #builder_name{
                let mut input = ::raiden::PutItemInput::default();
                let mut attribute_names: std::collections::HashMap<String, String> = std::collections::HashMap::new();
                let mut attribute_values: std::collections::HashMap<String, raiden::AttributeValue> = std::collections::HashMap::new();
                let mut uuid_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();

                #input_items

                let output_item = #item_output_name {
                    #(#output_values)*
                };
                input.item = input_item;
                // input.condition_expression = Some(":value0 = #name".to_owned());
                // input.condition_expression = Some("attribute_not_exists(#name) AND (attribute_not_exists(id) OR NOT attribute_not_exists(id))".to_owned());
                // input.condition_expression = Some("attribute_not_exists(id) AND NOT attribute_not_exists(id)".to_owned());
                // input.condition_expression = Some("attribute_not_exists(id) AND attribute_not_exists(id)".to_owned());
                // input.condition_expression = Some("attribute_not_exists(name)".to_owned());

                // #attribute_names
                // attribute_names.insert("#name".to_owned(), "name".to_owned());
                //  attribute_values.insert(":value0".to_owned(),  raiden::AttributeValue {
                //     s: Some("bokuweb".to_owned()),
                //     ..raiden::AttributeValue::default()
                //  });
                // attribute_values.insert(":test".to_owned(),  raiden::AttributeValue {
                //     n: Some("10".to_owned()),
                //     ..raiden::AttributeValue::default()
                // });
                // input.expression_attribute_names = Some(attribute_names);
                // input.expression_attribute_values = Some(attribute_values);
                input.table_name = self.table_name();
                #builder_name {
                    client: &self.client,
                    input,
                    item: output_item,
                }
            }
        }

        pub struct #builder_name<'a> {
            pub client: &'a ::raiden::DynamoDbClient,
            pub input: ::raiden::PutItemInput,
            pub item: #item_output_name,
        }

        impl<'a> #builder_name<'a> {

            fn raw_input(mut self, input: ::raiden::PutItemInput) -> Self {
                self.input = input;
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

            async fn run(self) -> Result<::raiden::put::PutOutput<#item_output_name>, ::raiden::RaidenError> {
                let res = self.client.put_item(self.input).await?;
                Ok(::raiden::put::PutOutput {
                    item: self.item,
                    consumed_capacity: res.consumed_capacity,
                })
            }
        }
    }
}

// https://docs.aws.amazon.com/ja_jp/amazondynamodb/latest/developerguide/Expressions.OperatorsAndFunctions.html

/*
/// <p>Represents the input of a <code>PutItem</code> operation.</p>
#[derive(Default, Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "deserialize_structs", derive(Deserialize))]
pub struct PutItemInput {
    /// <p>A condition that must be satisfied in order for a conditional <code>PutItem</code> operation to succeed.</p> <p>An expression can contain any of the following:</p> <ul> <li> <p>Functions: <code>attribute_exists | attribute_not_exists | attribute_type | contains | begins_with | size</code> </p> <p>These function names are case-sensitive.</p> </li> <li> <p>Comparison operators: <code>= | &lt;&gt; | &lt; | &gt; | &lt;= | &gt;= | BETWEEN | IN </code> </p> </li> <li> <p> Logical operators: <code>AND | OR | NOT</code> </p> </li> </ul> <p>For more information on condition expressions, see <a href="https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Expressions.SpecifyingConditions.html">Condition Expressions</a> in the <i>Amazon DynamoDB Developer Guide</i>.</p>
    #[serde(rename = "ConditionExpression")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition_expression: Option<String>,
    /// <p>This is a legacy parameter. Use <code>ConditionExpression</code> instead. For more information, see <a href="https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/LegacyConditionalParameters.ConditionalOperator.html">ConditionalOperator</a> in the <i>Amazon DynamoDB Developer Guide</i>.</p>
    #[serde(rename = "ConditionalOperator")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditional_operator: Option<String>,
    /// <p>This is a legacy parameter. Use <code>ConditionExpression</code> instead. For more information, see <a href="https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/LegacyConditionalParameters.Expected.html">Expected</a> in the <i>Amazon DynamoDB Developer Guide</i>.</p>
    #[serde(rename = "Expected")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<::std::collections::HashMap<String, ExpectedAttributeValue>>,
    /// <p>One or more substitution tokens for attribute names in an expression. The following are some use cases for using <code>ExpressionAttributeNames</code>:</p> <ul> <li> <p>To access an attribute whose name conflicts with a DynamoDB reserved word.</p> </li> <li> <p>To create a placeholder for repeating occurrences of an attribute name in an expression.</p> </li> <li> <p>To prevent special characters in an attribute name from being misinterpreted in an expression.</p> </li> </ul> <p>Use the <b>#</b> character in an expression to dereference an attribute name. For example, consider the following attribute name:</p> <ul> <li> <p> <code>Percentile</code> </p> </li> </ul> <p>The name of this attribute conflicts with a reserved word, so it cannot be used directly in an expression. (For the complete list of reserved words, see <a href="https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/ReservedWords.html">Reserved Words</a> in the <i>Amazon DynamoDB Developer Guide</i>). To work around this, you could specify the following for <code>ExpressionAttributeNames</code>:</p> <ul> <li> <p> <code>{"#P":"Percentile"}</code> </p> </li> </ul> <p>You could then use this substitution in an expression, as in this example:</p> <ul> <li> <p> <code>#P = :val</code> </p> </li> </ul> <note> <p>Tokens that begin with the <b>:</b> character are <i>expression attribute values</i>, which are placeholders for the actual value at runtime.</p> </note> <p>For more information on expression attribute names, see <a href="https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Expressions.AccessingItemAttributes.html">Specifying Item Attributes</a> in the <i>Amazon DynamoDB Developer Guide</i>.</p>
    #[serde(rename = "ExpressionAttributeNames")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expression_attribute_names: Option<::std::collections::HashMap<String, String>>,
    /// <p>One or more values that can be substituted in an expression.</p> <p>Use the <b>:</b> (colon) character in an expression to dereference an attribute value. For example, suppose that you wanted to check whether the value of the <i>ProductStatus</i> attribute was one of the following: </p> <p> <code>Available | Backordered | Discontinued</code> </p> <p>You would first need to specify <code>ExpressionAttributeValues</code> as follows:</p> <p> <code>{ ":avail":{"S":"Available"}, ":back":{"S":"Backordered"}, ":disc":{"S":"Discontinued"} }</code> </p> <p>You could then use these values in an expression, such as this:</p> <p> <code>ProductStatus IN (:avail, :back, :disc)</code> </p> <p>For more information on expression attribute values, see <a href="https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Expressions.SpecifyingConditions.html">Condition Expressions</a> in the <i>Amazon DynamoDB Developer Guide</i>.</p>
    #[serde(rename = "ExpressionAttributeValues")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expression_attribute_values: Option<::std::collections::HashMap<String, AttributeValue>>,
    /// <p>A map of attribute name/value pairs, one for each attribute. Only the primary key attributes are required; you can optionally provide other attribute name-value pairs for the item.</p> <p>You must provide all of the attributes for the primary key. For example, with a simple primary key, you only need to provide a value for the partition key. For a composite primary key, you must provide both values for both the partition key and the sort key.</p> <p>If you specify any attributes that are part of an index key, then the data types for those attributes must match those of the schema in the table's attribute definition.</p> <p>For more information about primary keys, see <a href="https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/HowItWorks.CoreComponents.html#HowItWorks.CoreComponents.PrimaryKey">Primary Key</a> in the <i>Amazon DynamoDB Developer Guide</i>.</p> <p>Each element in the <code>Item</code> map is an <code>AttributeValue</code> object.</p>
    #[serde(rename = "Item")]
    pub item: ::std::collections::HashMap<String, AttributeValue>,
    #[serde(rename = "ReturnConsumedCapacity")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_consumed_capacity: Option<String>,
    /// <p>Determines whether item collection metrics are returned. If set to <code>SIZE</code>, the response includes statistics about item collections, if any, that were modified during the operation are returned in the response. If set to <code>NONE</code> (the default), no statistics are returned.</p>
    #[serde(rename = "ReturnItemCollectionMetrics")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_item_collection_metrics: Option<String>,
    /// <p><p>Use <code>ReturnValues</code> if you want to get the item attributes as they appeared before they were updated with the <code>PutItem</code> request. For <code>PutItem</code>, the valid values are:</p> <ul> <li> <p> <code>NONE</code> - If <code>ReturnValues</code> is not specified, or if its value is <code>NONE</code>, then nothing is returned. (This setting is the default for <code>ReturnValues</code>.)</p> </li> <li> <p> <code>ALL<em>OLD</code> - If <code>PutItem</code> overwrote an attribute name-value pair, then the content of the old item is returned.</p> </li> </ul> <note> <p>The <code>ReturnValues</code> parameter is used by several DynamoDB operations; however, <code>PutItem</code> does not recognize any values other than <code>NONE</code> or <code>ALL</em>OLD</code>.</p> </note></p>
    #[serde(rename = "ReturnValues")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_values: Option<String>,
    /// <p>The name of the table to contain the item.</p>
    #[serde(rename = "TableName")]
    pub table_name: String,
}

*/
