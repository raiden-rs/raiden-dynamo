#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
use crate::ConsumedCapacity;

#[cfg(feature = "aws-sdk")]
use crate::aws_sdk::types::ConsumedCapacity;

#[derive(Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    any(feature = "rusoto", feature = "rusoto_rustls"),
    derive(serde::Deserialize, serde::Serialize)
)]
pub struct TransactGetOutput<T> {
    pub consumed_capacity: Option<Vec<ConsumedCapacity>>,
    pub items: Vec<Option<T>>,
}
