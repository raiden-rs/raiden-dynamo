#[cfg(feature = "rusoto")]
use crate::{ConsumedCapacity, ItemCollectionMetrics};

#[cfg(feature = "aws-sdk")]
use crate::aws_sdk::types::{ConsumedCapacity, ItemCollectionMetrics};

// See. https://github.com/rusoto/rusoto/blob/cf22a4348ae717a20760bb9934cfd118ddb4437e/rusoto/services/dynamodb/src/generated.rs#L2971
#[derive(Default, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "rusoto", derive(serde::Deserialize, serde::Serialize))]
pub struct UpdateOutput<T> {
    pub consumed_capacity: Option<ConsumedCapacity>,
    pub item: Option<T>,
    #[cfg_attr(
        feature = "rusoto",
        serde(
            default,
            deserialize_with = "crate::attribute_json::item_collection_metrics"
        )
    )]
    pub item_collection_metrics: Option<ItemCollectionMetrics>,
}
