use proc_macro2::*;
use quote::*;
use syn::*;

pub(crate) fn expand_delete_item(
    partition_key: &(Ident, Type),
    sort_key: &Option<(Ident, Type)>,
    struct_name: &Ident,
) -> TokenStream {
    let trait_name = format_ident!("{}DeleteItem", struct_name);
    let client_name = format_ident!("{}Client", struct_name);
    let builder_name = format_ident!("{}DeleteItemBuilder", struct_name);
    let condition_token_name = format_ident!("{}ConditionToken", struct_name);
    let (partition_key_ident, partition_key_type) = partition_key;

    let client_trait = if let Some(sort_key) = sort_key {
        let (sort_key_ident, sort_key_type) = sort_key;
        quote! {
            pub trait #trait_name {
                fn delete(&self, pk: impl Into<#partition_key_type>, sk: impl Into<#sort_key_type>) -> #builder_name;
            }

            impl #trait_name for #client_name {
                fn delete(&self, pk: impl Into<#partition_key_type>, sk: impl Into<#sort_key_type>) -> #builder_name {
                    let mut input = ::raiden::DeleteItemInput::default();
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
                    }
                }
            }
        }
    } else {
        quote! {
            pub trait #trait_name {
                fn delete(&self, key: impl Into<#partition_key_type>) -> #builder_name;
            }

            impl #trait_name for #client_name {
                fn delete(&self, key: impl Into<#partition_key_type>) -> #builder_name {
                    let mut input = ::raiden::DeleteItemInput::default();
                    let key_attr: AttributeValue = key.into().into_attr();
                    let mut key_set: std::collections::HashMap<String, AttributeValue> = std::collections::HashMap::new();
                    key_set.insert(stringify!(#partition_key_ident).to_owned(), key_attr);
                    input.key = key_set;
                    input.table_name = self.table_name();
                    #builder_name {
                        client: &self.client,
                        input,
                    }
                }
            }
        }
    };

    quote! {
        #client_trait

        pub struct #builder_name<'a> {
            pub client: &'a ::raiden::DynamoDbClient,
            pub input: ::raiden::DeleteItemInput,
        }

        impl<'a> #builder_name<'a> {
            pub fn raw_input(mut self, input: ::raiden::DeleteItemInput) -> Self {
                self.input = input;
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

            async fn run(self) -> Result<(), ::raiden::RaidenError> {
                let res = self.client.delete_item(self.input).await?;
                Ok(())
            }
        }
    }
}

/*
#[derive(Default, Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "deserialize_structs", derive(Deserialize))]
pub struct DeleteItemInput {
    /// <p>A condition that must be satisfied in order for a conditional <code>DeleteItem</code> to succeed.</p> <p>An expression can contain any of the following:</p> <ul> <li> <p>Functions: <code>attribute_exists | attribute_not_exists | attribute_type | contains | begins_with | size</code> </p> <p>These function names are case-sensitive.</p> </li> <li> <p>Comparison operators: <code>= | &lt;&gt; | &lt; | &gt; | &lt;= | &gt;= | BETWEEN | IN </code> </p> </li> <li> <p> Logical operators: <code>AND | OR | NOT</code> </p> </li> </ul> <p>For more information about condition expressions, see <a href="https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Expressions.SpecifyingConditions.html">Condition Expressions</a> in the <i>Amazon DynamoDB Developer Guide</i>.</p>
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
    /// <p>A map of attribute names to <code>AttributeValue</code> objects, representing the primary key of the item to delete.</p> <p>For the primary key, you must provide all of the attributes. For example, with a simple primary key, you only need to provide a value for the partition key. For a composite primary key, you must provide values for both the partition key and the sort key.</p>
    #[serde(rename = "Key")]
    pub key: ::std::collections::HashMap<String, AttributeValue>,
    #[serde(rename = "ReturnConsumedCapacity")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_consumed_capacity: Option<String>,
    /// <p>Determines whether item collection metrics are returned. If set to <code>SIZE</code>, the response includes statistics about item collections, if any, that were modified during the operation are returned in the response. If set to <code>NONE</code> (the default), no statistics are returned.</p>
    #[serde(rename = "ReturnItemCollectionMetrics")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_item_collection_metrics: Option<String>,
    /// <p><p>Use <code>ReturnValues</code> if you want to get the item attributes as they appeared before they were deleted. For <code>DeleteItem</code>, the valid values are:</p> <ul> <li> <p> <code>NONE</code> - If <code>ReturnValues</code> is not specified, or if its value is <code>NONE</code>, then nothing is returned. (This setting is the default for <code>ReturnValues</code>.)</p> </li> <li> <p> <code>ALL<em>OLD</code> - The content of the old item is returned.</p> </li> </ul> <note> <p>The <code>ReturnValues</code> parameter is used by several DynamoDB operations; however, <code>DeleteItem</code> does not recognize any values other than <code>NONE</code> or <code>ALL</em>OLD</code>.</p> </note></p>
    #[serde(rename = "ReturnValues")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_values: Option<String>,
    /// <p>The name of the table from which to delete the item.</p>
    #[serde(rename = "TableName")]
    pub table_name: String,
}

*/
