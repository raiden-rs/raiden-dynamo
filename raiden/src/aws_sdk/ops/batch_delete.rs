use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};
use serde::ser::{Serialize, SerializeStruct, Serializer};

use crate::ops::batch_delete::BatchDeleteOutput;

impl Serialize for BatchDeleteOutput {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("BatchDeleteOutput", 2)?;
        state.serialize_field(
            "consumed_capacity",
            &self.consumed_capacity.as_ref().map(|v| {
                v.iter()
                    .map(crate::aws_sdk::serialize::consumed_capacity_to_value)
                    .collect::<Vec<_>>()
            }),
        )?;
        state.serialize_field(
            "unprocessed_items",
            &self
                .unprocessed_items
                .iter()
                .map(crate::aws_sdk::serialize::delete_request_to_value)
                .collect::<Vec<_>>(),
        )?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for BatchDeleteOutput {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            ConsumedCapacity,
            UnprocessedItems,
        }

        const FIELDS: &[&str] = &["consumed_capacity", "unprocessed_items"];

        struct BatchDeleteOutputVisitor;

        impl<'de> Visitor<'de> for BatchDeleteOutputVisitor {
            type Value = BatchDeleteOutput;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct BatchDeleteOutput")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut consumed_capacity = None;
                let mut unprocessed_items = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::ConsumedCapacity => {
                            if consumed_capacity.is_some() {
                                return Err(de::Error::duplicate_field("consumed_capacity"));
                            }

                            let vs: Option<Vec<serde_json::Value>> = map.next_value()?;

                            consumed_capacity = if let Some(vs) = vs {
                                let mut values = vec![];

                                for v in vs {
                                    values.push(
                                        crate::aws_sdk::serialize::value_to_consumed_capacity(v)
                                            .map_err(de::Error::custom)?,
                                    );
                                }

                                Some(values)
                            } else {
                                None
                            };
                        }
                        Field::UnprocessedItems => {
                            if unprocessed_items.is_some() {
                                return Err(de::Error::duplicate_field("unprocessed_items"));
                            }

                            let vs: Option<Vec<serde_json::Value>> = map.next_value()?;

                            unprocessed_items = if let Some(vs) = vs {
                                let mut values = vec![];

                                for v in vs {
                                    values.push(
                                        crate::aws_sdk::serialize::value_to_delete_request(v)
                                            .map_err(de::Error::custom)?,
                                    );
                                }

                                Some(values)
                            } else {
                                None
                            };
                        }
                    }
                }

                let unprocessed_items = unprocessed_items
                    .ok_or_else(|| de::Error::missing_field("unprocessed_items"))?;

                Ok(BatchDeleteOutput {
                    consumed_capacity,
                    unprocessed_items,
                })
            }
        }

        deserializer.deserialize_struct("BatchDeleteOutput", FIELDS, BatchDeleteOutputVisitor)
    }
}
