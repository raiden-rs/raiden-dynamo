#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
use crate::{ConsumedCapacity, DeleteRequest};

#[cfg(feature = "aws-sdk")]
use crate::aws_sdk::types::{ConsumedCapacity, DeleteRequest};

// See. https://github.com/rusoto/rusoto/blob/69e7c9150d98916ef8fc814f5cd17eb0e4dee3d3/rusoto/services/dynamodb/src/generated.rs#L395
#[derive(Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    any(feature = "rusoto", feature = "rusoto_rustls"),
    derive(serde::Deserialize, serde::Serialize)
)]
pub struct BatchDeleteOutput {
    pub consumed_capacity: Option<Vec<ConsumedCapacity>>,
    #[cfg_attr(
        any(feature = "rusoto", feature = "rusoto_rustls"),
        serde(deserialize_with = "crate::attribute_json::delete_requests")
    )]
    pub unprocessed_items: Vec<DeleteRequest>,
}
