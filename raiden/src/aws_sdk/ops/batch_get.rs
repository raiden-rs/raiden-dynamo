use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};
use serde::ser::{Serialize, SerializeStruct, Serializer};

use crate::ops::batch_get::BatchGetOutput;

impl<T> Serialize for BatchGetOutput<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("BatchGetOutput", 3)?;
        state.serialize_field(
            "consumed_capacity",
            &self.consumed_capacity.as_ref().map(|v| {
                v.iter()
                    .map(|v| crate::aws_sdk::serialize::consumed_capacity_to_value(&v))
                    .collect::<Vec<_>>()
            }),
        )?;
        state.serialize_field("items", &self.items)?;
        state.serialize_field(
            "unprocessed_keys",
            &self
                .unprocessed_keys
                .as_ref()
                .map(|v| crate::aws_sdk::serialize::keys_and_attributes_to_value(&v)),
        )?;
        state.end()
    }
}

impl<'de, T> Deserialize<'de> for BatchGetOutput<T>
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
            Items,
            UnprocessedKeys,
        }

        const FIELDS: &'static [&'static str] = &["consumed_capacity", "items", "unprocessed_keys"];

        struct BatchGetOutputVisitor<'de, T>
        where
            T: Deserialize<'de>,
        {
            marker: std::marker::PhantomData<BatchGetOutput<T>>,
            lifetime: std::marker::PhantomData<&'de ()>,
        }

        impl<'de, T> Visitor<'de> for BatchGetOutputVisitor<'de, T>
        where
            T: Deserialize<'de>,
        {
            type Value = BatchGetOutput<T>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct BatchGetOutput")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut consumed_capacity = None;
                let mut items = None;
                let mut unprocessed_keys = None;

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
                        Field::Items => {
                            if items.is_some() {
                                return Err(de::Error::duplicate_field("items"));
                            }

                            items = Some(map.next_value()?);
                        }
                        Field::UnprocessedKeys => {
                            if unprocessed_keys.is_some() {
                                return Err(de::Error::duplicate_field("unprocessed_keys"));
                            }

                            let v: Option<serde_json::Value> = map.next_value()?;

                            unprocessed_keys = if let Some(v) = v {
                                Some(
                                    crate::aws_sdk::serialize::value_to_keys_and_attributes(v)
                                        .map_err(de::Error::custom)?,
                                )
                            } else {
                                None
                            };
                        }
                    }
                }

                let items = items.ok_or_else(|| de::Error::missing_field("items"))?;

                Ok(BatchGetOutput {
                    consumed_capacity,
                    items,
                    unprocessed_keys,
                })
            }
        }

        deserializer.deserialize_struct(
            "BatchGetOutput",
            FIELDS,
            BatchGetOutputVisitor {
                marker: std::marker::PhantomData::<BatchGetOutput<T>>,
                lifetime: std::marker::PhantomData,
            },
        )
    }
}
