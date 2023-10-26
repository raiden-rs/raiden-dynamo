// See. https://github.com/rusoto/rusoto/blob/cf22a4348ae717a20760bb9934cfd118ddb4437e/rusoto/services/dynamodb/src/generated.rs#L1168
#[derive(Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    any(feature = "rusoto", feature = "rusoto_rustls"),
    derive(serde::Deserialize, serde::Serialize)
)]
pub struct PutOutput<T> {
    pub consumed_capacity: Option<crate::ConsumedCapacity>,
    pub item: T,
}
