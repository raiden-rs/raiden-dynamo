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
pub struct GetOutput<T> {
    pub consumed_capacity: Option<ConsumedCapacity>,
    pub item: T,
}
