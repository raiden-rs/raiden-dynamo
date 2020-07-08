use serde::{Deserialize, Serialize};

// See. https://github.com/rusoto/rusoto/blob/cf22a4348ae717a20760bb9934cfd118ddb4437e/rusoto/services/dynamodb/src/generated.rs#L2971
#[derive(Default, Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct UpdateOutput<T> {
    pub consumed_capacity: Option<crate::ConsumedCapacity>,
    pub item: T,
    pub item_collection_metrics: Option<crate::ItemCollectionMetrics>,
}
