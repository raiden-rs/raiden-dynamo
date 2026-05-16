use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, SerializeStruct, Serializer};

use crate::ops::transact_get::TransactGetOutput;

impl<T> Serialize for TransactGetOutput<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("TransactGetOutput", 2)?;
        state.serialize_field(
            "consumed_capacity",
            &self.consumed_capacity.as_ref().map(|v| {
                v.iter()
                    .map(crate::aws_sdk::serialize::consumed_capacity_to_value)
                    .collect::<Vec<_>>()
            }),
        )?;
        state.serialize_field("items", &self.items)?;
        state.end()
    }
}

impl<'de, T> Deserialize<'de> for TransactGetOutput<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct Helper<T> {
            consumed_capacity: Option<Vec<serde_json::Value>>,
            items: Vec<Option<T>>,
        }

        let helper = Helper::deserialize(deserializer)?;
        let consumed_capacity = helper
            .consumed_capacity
            .map(|values| {
                values
                    .into_iter()
                    .map(crate::aws_sdk::serialize::value_to_consumed_capacity)
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()
            .map_err(serde::de::Error::custom)?;

        Ok(TransactGetOutput {
            consumed_capacity,
            items: helper.items,
        })
    }
}
