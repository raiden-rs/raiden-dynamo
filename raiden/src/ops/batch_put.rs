#[cfg(feature = "rusoto")]
use crate::{ConsumedCapacity, PutRequest};

#[cfg(feature = "aws-sdk")]
use crate::aws_sdk::types::{ConsumedCapacity, PutRequest};

#[derive(Default, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "rusoto", derive(serde::Deserialize, serde::Serialize))]
pub struct BatchPutOutput {
    pub consumed_capacity: Option<Vec<ConsumedCapacity>>,
    #[cfg_attr(
        feature = "rusoto",
        serde(deserialize_with = "crate::attribute_json::put_requests")
    )]
    pub unprocessed_items: Vec<PutRequest>,
}
