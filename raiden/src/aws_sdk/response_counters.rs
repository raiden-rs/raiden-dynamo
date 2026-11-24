use std::sync::{Arc, Mutex};

use aws_smithy_runtime_api::{
    box_error::BoxError,
    client::{
        interceptors::{context::AfterDeserializationInterceptorContextRef, Intercept},
        runtime_components::RuntimeComponents,
    },
};

// Query/Scan's SDK output defaults missing integer fields to zero. Retain their
// original presence so Raiden can preserve Rusoto's Option<i64> output contract.
#[doc(hidden)]
#[derive(Clone, Debug, Default)]
pub struct ResponseCounters(Arc<Mutex<Counters>>);

#[derive(Debug, Default, serde::Deserialize)]
struct Counters {
    #[serde(rename = "Count")]
    count: Option<i64>,
    #[serde(rename = "ScannedCount")]
    scanned_count: Option<i64>,
}

impl ResponseCounters {
    pub async fn query(
        client: &crate::Client,
        input: crate::aws_sdk::operation::query::builders::QueryInputBuilder,
        counters: Self,
    ) -> Result<
        crate::aws_sdk::operation::query::QueryOutput,
        crate::aws_sdk::error::SdkError<crate::aws_sdk::operation::query::QueryError>,
    > {
        // Transfer every input field while retaining the original client's
        // connection pool, runtime plugins and retry state.
        client
            .query()
            .set_table_name(input.get_table_name().clone())
            .set_index_name(input.get_index_name().clone())
            .set_select(input.get_select().clone())
            .set_attributes_to_get(input.get_attributes_to_get().clone())
            .set_limit(input.get_limit().clone())
            .set_consistent_read(input.get_consistent_read().clone())
            .set_key_conditions(input.get_key_conditions().clone())
            .set_query_filter(input.get_query_filter().clone())
            .set_conditional_operator(input.get_conditional_operator().clone())
            .set_scan_index_forward(input.get_scan_index_forward().clone())
            .set_exclusive_start_key(input.get_exclusive_start_key().clone())
            .set_return_consumed_capacity(input.get_return_consumed_capacity().clone())
            .set_projection_expression(input.get_projection_expression().clone())
            .set_filter_expression(input.get_filter_expression().clone())
            .set_key_condition_expression(input.get_key_condition_expression().clone())
            .set_expression_attribute_names(input.get_expression_attribute_names().clone())
            .set_expression_attribute_values(input.get_expression_attribute_values().clone())
            .customize()
            .interceptor(counters)
            .send()
            .await
    }

    pub async fn scan(
        client: &crate::Client,
        input: crate::aws_sdk::operation::scan::builders::ScanInputBuilder,
        counters: Self,
    ) -> Result<
        crate::aws_sdk::operation::scan::ScanOutput,
        crate::aws_sdk::error::SdkError<crate::aws_sdk::operation::scan::ScanError>,
    > {
        // Transfer every input field while retaining the original client's
        // connection pool, runtime plugins and retry state.
        client
            .scan()
            .set_table_name(input.get_table_name().clone())
            .set_index_name(input.get_index_name().clone())
            .set_attributes_to_get(input.get_attributes_to_get().clone())
            .set_limit(input.get_limit().clone())
            .set_select(input.get_select().clone())
            .set_scan_filter(input.get_scan_filter().clone())
            .set_conditional_operator(input.get_conditional_operator().clone())
            .set_exclusive_start_key(input.get_exclusive_start_key().clone())
            .set_return_consumed_capacity(input.get_return_consumed_capacity().clone())
            .set_total_segments(input.get_total_segments().clone())
            .set_segment(input.get_segment().clone())
            .set_projection_expression(input.get_projection_expression().clone())
            .set_filter_expression(input.get_filter_expression().clone())
            .set_expression_attribute_names(input.get_expression_attribute_names().clone())
            .set_expression_attribute_values(input.get_expression_attribute_values().clone())
            .set_consistent_read(input.get_consistent_read().clone())
            .customize()
            .interceptor(counters)
            .send()
            .await
    }

    pub fn get(&self) -> (Option<i64>, Option<i64>) {
        let counters = self.0.lock().expect("response counters lock poisoned");
        (counters.count, counters.scanned_count)
    }
}

impl Intercept for ResponseCounters {
    fn name(&self) -> &'static str {
        "RaidenResponseCounters"
    }

    fn read_after_deserialization(
        &self,
        context: &AfterDeserializationInterceptorContextRef<'_>,
        _runtime_components: &RuntimeComponents,
        _cfg: &mut aws_smithy_types::config_bag::ConfigBag,
    ) -> Result<(), BoxError> {
        let counters = context
            .response()
            .body()
            .bytes()
            .and_then(|bytes| serde_json::from_slice(bytes).ok())
            .unwrap_or_default();
        *self.0.lock().expect("response counters lock poisoned") = counters;
        Ok(())
    }
}
