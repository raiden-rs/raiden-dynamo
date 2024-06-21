use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};
use serde::ser::{Serialize, SerializeStruct, Serializer};

use crate::ops::update::UpdateOutput;

impl<T> Serialize for UpdateOutput<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("UpdateOutput", 3)?;
        state.serialize_field(
            "consumed_capacity",
            &self
                .consumed_capacity
                .as_ref()
                .map(crate::aws_sdk::serialize::consumed_capacity_to_value),
        )?;
        state.serialize_field("item", &self.item)?;
        state.serialize_field(
            "item_collection_metrics",
            &self
                .item_collection_metrics
                .as_ref()
                .map(crate::aws_sdk::serialize::item_collection_metrics_to_value),
        )?;
        state.end()
    }
}

impl<'de, T> Deserialize<'de> for UpdateOutput<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            ConsumedCapacity,
            Item,
            ItemCollectionMetrics,
        }

        const FIELDS: &[&str] = &["consumed_capacity", "item", "item_collection_metrics"];

        struct UpdateOutputVisitor<'de, T>
        where
            T: Deserialize<'de>,
        {
            marker: std::marker::PhantomData<UpdateOutput<T>>,
            lifetime: std::marker::PhantomData<&'de ()>,
        }

        impl<'de, T> Visitor<'de> for UpdateOutputVisitor<'de, T>
        where
            T: Deserialize<'de>,
        {
            type Value = UpdateOutput<T>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct UpdateOutput")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut consumed_capacity = None;
                let mut item = None;
                let mut item_collection_metrics = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::ConsumedCapacity => {
                            if consumed_capacity.is_some() {
                                return Err(de::Error::duplicate_field("consumed_capacity"));
                            }

                            let v: Option<serde_json::Value> = map.next_value()?;

                            consumed_capacity = if let Some(v) = v {
                                Some(
                                    crate::aws_sdk::serialize::value_to_consumed_capacity(v)
                                        .map_err(de::Error::custom)?,
                                )
                            } else {
                                None
                            };
                        }
                        Field::Item => {
                            if item.is_some() {
                                return Err(de::Error::duplicate_field("item"));
                            }

                            item = map.next_value()?;
                        }
                        Field::ItemCollectionMetrics => {
                            if item_collection_metrics.is_some() {
                                return Err(de::Error::duplicate_field("item_collection_metrics"));
                            }

                            let v: Option<serde_json::Value> = map.next_value()?;

                            item_collection_metrics = if let Some(v) = v {
                                Some(
                                    crate::aws_sdk::serialize::value_to_item_collection_metrics(v)
                                        .map_err(de::Error::custom)?,
                                )
                            } else {
                                None
                            };
                        }
                    }
                }

                Ok(UpdateOutput {
                    consumed_capacity,
                    item,
                    item_collection_metrics,
                })
            }
        }

        deserializer.deserialize_struct(
            "UpdateOutput",
            FIELDS,
            UpdateOutputVisitor {
                marker: std::marker::PhantomData::<UpdateOutput<T>>,
                lifetime: std::marker::PhantomData,
            },
        )
    }
}
