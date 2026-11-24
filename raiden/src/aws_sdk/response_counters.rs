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
    pub fn client(&self, client: &crate::Client) -> crate::Client {
        crate::Client::from_conf(
            client
                .config()
                .to_builder()
                .interceptor(self.clone())
                .build(),
        )
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
