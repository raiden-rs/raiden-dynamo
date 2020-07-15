use serde::{Deserialize, Serialize};

// See. https://github.com/rusoto/rusoto/blob/cf22a4348ae717a20760bb9934cfd118ddb4437e/rusoto/services/dynamodb/src/generated.rs#L2406
#[derive(Default, Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ScanOutput<T> {
    pub consumed_capacity: Option<crate::ConsumedCapacity>,
    pub items: Vec<T>,
    pub count: Option<i64>,
    pub last_evaluated_key: Option<::std::collections::HashMap<String, crate::AttributeValue>>,
    pub scanned_count: Option<i64>,
}
