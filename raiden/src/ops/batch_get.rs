use serde::{Deserialize, Serialize};

// See. https://github.com/rusoto/rusoto/blob/69e7c9150d98916ef8fc814f5cd17eb0e4dee3d3/rusoto/services/dynamodb/src/generated.rs#L356
#[derive(Default, Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct BatchGetOutput<T> {
    pub consumed_capacity: Option<Vec<crate::ConsumedCapacity>>,
    pub items: Vec<T>,
    pub unprocessed_keys: ::std::collections::HashMap<String, crate::KeysAndAttributes>,
}
