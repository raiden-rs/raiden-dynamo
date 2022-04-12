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
    let from_item = super::expand_attr_to_item(&format_ident!("res_item"), fields, rename_all_type);

    quote! {
        pub trait #trait_name {
            fn scan(&self) -> #builder_name;
        }

        pub struct #builder_name<'a> {
            pub client: &'a ::raiden::DynamoDbClient,
            pub input: ::raiden::ScanInput,
            pub next_token: Option<::raiden::NextToken>,
            pub limit: Option<i64>
        }

        impl #trait_name for #client_name {
            #![allow(clippy::field_reassign_with_default)]
            fn scan(&self) -> #builder_name {
                let mut input = ::raiden::ScanInput::default();
                input.table_name = self.table_name();
                input.projection_expression = self.projection_expression.clone();
                input.expression_attribute_names = self.attribute_names.clone();
                #builder_name {
                    client: &self.client,
                    input,
                    next_token: None,
                    limit: None,
                }
            }
        }

        impl<'a> #builder_name<'a> {
            pub fn index(mut self, index: impl Into<String>) -> Self {
                self.input.index_name = Some(index.into());
                self
            }

            pub fn consistent(mut self) -> Self {
                self.input.consistent_read = Some(true);
                self
            }

            pub fn filter(mut self, cond: impl ::raiden::filter_expression::FilterExpressionBuilder<#filter_expression_token_name>) -> Self {
                let (cond_str, attr_names, attr_values) = cond.build();
                if !attr_values.is_empty() {
                    if let Some(v) = self.input.expression_attribute_values {
                        self.input.expression_attribute_values = Some(::raiden::merge_map(attr_values, v));
                    } else {
                        self.input.expression_attribute_values = Some(attr_values);
                    }
                }
                self.input.filter_expression = Some(cond_str);
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
                    self.input.exclusive_start_key = Some(token.into_attr_values()?);
                }

                let mut items: Vec<#struct_name> = vec![];

                loop {
                    if let Some(limit) = self.limit {
                        self.input.limit = Some(limit);
                    }

                    let res = self.client.scan(self.input.clone()).await?;
                    if let Some(res_items) = res.items {
                        for res_item in res_items.into_iter() {
                            items.push(#struct_name {
                                #(#from_item)*
                            })
                        }
                    };

                    let scanned = &res.scanned_count.unwrap_or(0);

                    let mut has_next = true;
                    if let Some(limit) = self.limit {
                        has_next = limit - scanned > 0;
                        self.limit = Some(limit - scanned);
                    }
                    if res.last_evaluated_key.is_none() || !has_next {
                        return Ok(::raiden::scan::ScanOutput {
                            consumed_capacity: res.consumed_capacity,
                            count: res.count,
                            items,
                            last_evaluated_key: res.last_evaluated_key,
                            scanned_count: res.scanned_count,
                        })
                    }
                    self.input.exclusive_start_key = res.last_evaluated_key;
                }
            }
        }
    }
}

/*
/// <p>Represents the input of a <code>Scan</code> operation.</p>
#[derive(Default, Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "deserialize_structs", derive(Deserialize))]
pub struct ScanInput {
    /// <p>This is a legacy parameter. Use <code>ProjectionExpression</code> instead. For more information, see <a href="https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/LegacyConditionalParameters.AttributesToGet.html">AttributesToGet</a> in the <i>Amazon DynamoDB Developer Guide</i>.</p>
    #[serde(rename = "AttributesToGet")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes_to_get: Option<Vec<String>>,
    /// <p>This is a legacy parameter. Use <code>FilterExpression</code> instead. For more information, see <a href="https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/LegacyConditionalParameters.ConditionalOperator.html">ConditionalOperator</a> in the <i>Amazon DynamoDB Developer Guide</i>.</p>
    #[serde(rename = "ConditionalOperator")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditional_operator: Option<String>,
    /// <p>A Boolean value that determines the read consistency model during the scan:</p> <ul> <li> <p>If <code>ConsistentRead</code> is <code>false</code>, then the data returned from <code>Scan</code> might not contain the results from other recently completed write operations (<code>PutItem</code>, <code>UpdateItem</code>, or <code>DeleteItem</code>).</p> </li> <li> <p>If <code>ConsistentRead</code> is <code>true</code>, then all of the write operations that completed before the <code>Scan</code> began are guaranteed to be contained in the <code>Scan</code> response.</p> </li> </ul> <p>The default setting for <code>ConsistentRead</code> is <code>false</code>.</p> <p>The <code>ConsistentRead</code> parameter is not supported on global secondary indexes. If you scan a global secondary index with <code>ConsistentRead</code> set to true, you will receive a <code>ValidationException</code>.</p>
    #[serde(rename = "ConsistentRead")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consistent_read: Option<bool>,
    /// <p>The primary key of the first item that this operation will evaluate. Use the value that was returned for <code>LastEvaluatedKey</code> in the previous operation.</p> <p>The data type for <code>ExclusiveStartKey</code> must be String, Number or Binary. No set data types are allowed.</p> <p>In a parallel scan, a <code>Scan</code> request that includes <code>ExclusiveStartKey</code> must specify the same segment whose previous <code>Scan</code> returned the corresponding value of <code>LastEvaluatedKey</code>.</p>
    #[serde(rename = "ExclusiveStartKey")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusive_start_key: Option<::std::collections::HashMap<String, AttributeValue>>,
    /// <p>One or more substitution tokens for attribute names in an expression. The following are some use cases for using <code>ExpressionAttributeNames</code>:</p> <ul> <li> <p>To access an attribute whose name conflicts with a DynamoDB reserved word.</p> </li> <li> <p>To create a placeholder for repeating occurrences of an attribute name in an expression.</p> </li> <li> <p>To prevent special characters in an attribute name from being misinterpreted in an expression.</p> </li> </ul> <p>Use the <b>#</b> character in an expression to dereference an attribute name. For example, consider the following attribute name:</p> <ul> <li> <p> <code>Percentile</code> </p> </li> </ul> <p>The name of this attribute conflicts with a reserved word, so it cannot be used directly in an expression. (For the complete list of reserved words, see <a href="https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/ReservedWords.html">Reserved Words</a> in the <i>Amazon DynamoDB Developer Guide</i>). To work around this, you could specify the following for <code>ExpressionAttributeNames</code>:</p> <ul> <li> <p> <code>{"#P":"Percentile"}</code> </p> </li> </ul> <p>You could then use this substitution in an expression, as in this example:</p> <ul> <li> <p> <code>#P = :val</code> </p> </li> </ul> <note> <p>Tokens that begin with the <b>:</b> character are <i>expression attribute values</i>, which are placeholders for the actual value at runtime.</p> </note> <p>For more information on expression attribute names, see <a href="https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Expressions.AccessingItemAttributes.html">Specifying Item Attributes</a> in the <i>Amazon DynamoDB Developer Guide</i>.</p>
    #[serde(rename = "ExpressionAttributeNames")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expression_attribute_names: Option<::std::collections::HashMap<String, String>>,
    /// <p>One or more values that can be substituted in an expression.</p> <p>Use the <b>:</b> (colon) character in an expression to dereference an attribute value. For example, suppose that you wanted to check whether the value of the <code>ProductStatus</code> attribute was one of the following: </p> <p> <code>Available | Backordered | Discontinued</code> </p> <p>You would first need to specify <code>ExpressionAttributeValues</code> as follows:</p> <p> <code>{ ":avail":{"S":"Available"}, ":back":{"S":"Backordered"}, ":disc":{"S":"Discontinued"} }</code> </p> <p>You could then use these values in an expression, such as this:</p> <p> <code>ProductStatus IN (:avail, :back, :disc)</code> </p> <p>For more information on expression attribute values, see <a href="https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Expressions.SpecifyingConditions.html">Condition Expressions</a> in the <i>Amazon DynamoDB Developer Guide</i>.</p>
    #[serde(rename = "ExpressionAttributeValues")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expression_attribute_values: Option<::std::collections::HashMap<String, AttributeValue>>,
    /// <p>A string that contains conditions that DynamoDB applies after the <code>Scan</code> operation, but before the data is returned to you. Items that do not satisfy the <code>FilterExpression</code> criteria are not returned.</p> <note> <p>A <code>FilterExpression</code> is applied after the items have already been read; the process of filtering does not consume any additional read capacity units.</p> </note> <p>For more information, see <a href="https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/QueryAndScan.html#FilteringResults">Filter Expressions</a> in the <i>Amazon DynamoDB Developer Guide</i>.</p>
    #[serde(rename = "FilterExpression")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_expression: Option<String>,
    /// <p>The name of a secondary index to scan. This index can be any local secondary index or global secondary index. Note that if you use the <code>IndexName</code> parameter, you must also provide <code>TableName</code>.</p>
    #[serde(rename = "IndexName")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index_name: Option<String>,
    /// <p>The maximum number of items to evaluate (not necessarily the number of matching items). If DynamoDB processes the number of items up to the limit while processing the results, it stops the operation and returns the matching values up to that point, and a key in <code>LastEvaluatedKey</code> to apply in a subsequent operation, so that you can pick up where you left off. Also, if the processed dataset size exceeds 1 MB before DynamoDB reaches this limit, it stops the operation and returns the matching values up to the limit, and a key in <code>LastEvaluatedKey</code> to apply in a subsequent operation to continue the operation. For more information, see <a href="https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/QueryAndScan.html">Working with Queries</a> in the <i>Amazon DynamoDB Developer Guide</i>.</p>
    #[serde(rename = "Limit")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// <p>A string that identifies one or more attributes to retrieve from the specified table or index. These attributes can include scalars, sets, or elements of a JSON document. The attributes in the expression must be separated by commas.</p> <p>If no attribute names are specified, then all attributes will be returned. If any of the requested attributes are not found, they will not appear in the result.</p> <p>For more information, see <a href="https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Expressions.AccessingItemAttributes.html">Specifying Item Attributes</a> in the <i>Amazon DynamoDB Developer Guide</i>.</p>
    #[serde(rename = "ProjectionExpression")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projection_expression: Option<String>,
    #[serde(rename = "ReturnConsumedCapacity")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_consumed_capacity: Option<String>,
    /// <p>This is a legacy parameter. Use <code>FilterExpression</code> instead. For more information, see <a href="https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/LegacyConditionalParameters.ScanFilter.html">ScanFilter</a> in the <i>Amazon DynamoDB Developer Guide</i>.</p>
    #[serde(rename = "ScanFilter")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scan_filter: Option<::std::collections::HashMap<String, Condition>>,
    /// <p>For a parallel <code>Scan</code> request, <code>Segment</code> identifies an individual segment to be scanned by an application worker.</p> <p>Segment IDs are zero-based, so the first segment is always 0. For example, if you want to use four application threads to scan a table or an index, then the first thread specifies a <code>Segment</code> value of 0, the second thread specifies 1, and so on.</p> <p>The value of <code>LastEvaluatedKey</code> returned from a parallel <code>Scan</code> request must be used as <code>ExclusiveStartKey</code> with the same segment ID in a subsequent <code>Scan</code> operation.</p> <p>The value for <code>Segment</code> must be greater than or equal to 0, and less than the value provided for <code>TotalSegments</code>.</p> <p>If you provide <code>Segment</code>, you must also provide <code>TotalSegments</code>.</p>
    #[serde(rename = "Segment")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segment: Option<i64>,
    /// <p><p>The attributes to be returned in the result. You can retrieve all item attributes, specific item attributes, the count of matching items, or in the case of an index, some or all of the attributes projected into the index.</p> <ul> <li> <p> <code>ALL<em>ATTRIBUTES</code> - Returns all of the item attributes from the specified table or index. If you query a local secondary index, then for each matching item in the index, DynamoDB fetches the entire item from the parent table. If the index is configured to project all item attributes, then all of the data can be obtained from the local secondary index, and no fetching is required.</p> </li> <li> <p> <code>ALL</em>PROJECTED<em>ATTRIBUTES</code> - Allowed only when querying an index. Retrieves all attributes that have been projected into the index. If the index is configured to project all attributes, this return value is equivalent to specifying <code>ALL</em>ATTRIBUTES</code>.</p> </li> <li> <p> <code>COUNT</code> - Returns the number of matching items, rather than the matching items themselves.</p> </li> <li> <p> <code>SPECIFIC<em>ATTRIBUTES</code> - Returns only the attributes listed in <code>AttributesToGet</code>. This return value is equivalent to specifying <code>AttributesToGet</code> without specifying any value for <code>Select</code>.</p> <p>If you query or scan a local secondary index and request only attributes that are projected into that index, the operation reads only the index and not the table. If any of the requested attributes are not projected into the local secondary index, DynamoDB fetches each of these attributes from the parent table. This extra fetching incurs additional throughput cost and latency.</p> <p>If you query or scan a global secondary index, you can only request attributes that are projected into the index. Global secondary index queries cannot fetch attributes from the parent table.</p> </li> </ul> <p>If neither <code>Select</code> nor <code>AttributesToGet</code> are specified, DynamoDB defaults to <code>ALL</em>ATTRIBUTES</code> when accessing a table, and <code>ALL<em>PROJECTED</em>ATTRIBUTES</code> when accessing an index. You cannot use both <code>Select</code> and <code>AttributesToGet</code> together in a single request, unless the value for <code>Select</code> is <code>SPECIFIC<em>ATTRIBUTES</code>. (This usage is equivalent to specifying <code>AttributesToGet</code> without any value for <code>Select</code>.)</p> <note> <p>If you use the <code>ProjectionExpression</code> parameter, then the value for <code>Select</code> can only be <code>SPECIFIC</em>ATTRIBUTES</code>. Any other value for <code>Select</code> will return an error.</p> </note></p>
    #[serde(rename = "Select")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub select: Option<String>,
    /// <p>The name of the table containing the requested items; or, if you provide <code>IndexName</code>, the name of the table to which that index belongs.</p>
    #[serde(rename = "TableName")]
    pub table_name: String,
    /// <p>For a parallel <code>Scan</code> request, <code>TotalSegments</code> represents the total number of segments into which the <code>Scan</code> operation will be divided. The value of <code>TotalSegments</code> corresponds to the number of application workers that will perform the parallel scan. For example, if you want to use four application threads to scan a table or an index, specify a <code>TotalSegments</code> value of 4.</p> <p>The value for <code>TotalSegments</code> must be greater than or equal to 1, and less than or equal to 1000000. If you specify a <code>TotalSegments</code> value of 1, the <code>Scan</code> operation will be sequential rather than parallel.</p> <p>If you specify <code>TotalSegments</code>, you must also specify <code>Segment</code>.</p>
    #[serde(rename = "TotalSegments")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_segments: Option<i64>,
}
*/
