#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
use crate::ConsumedCapacity;

#[cfg(feature = "aws-sdk")]
use crate::aws_sdk::types::ConsumedCapacity;

// See. https://github.com/rusoto/rusoto/blob/cf22a4348ae717a20760bb9934cfd118ddb4437e/rusoto/services/dynamodb/src/generated.rs#L1168
#[derive(Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    any(feature = "rusoto", feature = "rusoto_rustls"),
    derive(serde::Deserialize, serde::Serialize)
)]
pub struct QueryOutput<T> {
    pub consumed_capacity: Option<ConsumedCapacity>,
    pub items: Vec<T>,
    pub count: Option<i64>,
    pub next_token: Option<crate::NextToken>,
    pub scanned_count: Option<i64>,
}
