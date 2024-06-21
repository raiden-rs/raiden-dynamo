#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
use crate::{ConsumedCapacity, KeysAndAttributes};

#[cfg(feature = "aws-sdk")]
use crate::aws_sdk::types::{ConsumedCapacity, KeysAndAttributes};

// See. https://github.com/rusoto/rusoto/blob/69e7c9150d98916ef8fc814f5cd17eb0e4dee3d3/rusoto/services/dynamodb/src/generated.rs#L356
#[derive(Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    any(feature = "rusoto", feature = "rusoto_rustls"),
    derive(serde::Deserialize, serde::Serialize)
)]
pub struct BatchGetOutput<T> {
    pub consumed_capacity: Option<Vec<ConsumedCapacity>>,
    pub items: Vec<T>,
    pub unprocessed_keys: Option<KeysAndAttributes>,
}
