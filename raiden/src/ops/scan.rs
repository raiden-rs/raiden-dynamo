#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
use crate::{AttributeValue, ConsumedCapacity};

#[cfg(feature = "aws-sdk")]
use crate::aws_sdk::types::{AttributeValue, ConsumedCapacity};

// See. https://github.com/rusoto/rusoto/blob/cf22a4348ae717a20760bb9934cfd118ddb4437e/rusoto/services/dynamodb/src/generated.rs#L2406
#[derive(Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    any(feature = "rusoto", feature = "rusoto_rustls"),
    derive(serde::Deserialize, serde::Serialize)
)]
pub struct ScanOutput<T> {
    pub consumed_capacity: Option<ConsumedCapacity>,
    pub items: Vec<T>,
    pub count: Option<i64>,
    pub last_evaluated_key: Option<::std::collections::HashMap<String, AttributeValue>>,
    pub scanned_count: Option<i64>,
}
