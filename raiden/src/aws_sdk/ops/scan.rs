use std::collections::HashMap;

use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};
use serde::ser::{Serialize, SerializeStruct, Serializer};

use crate::ops::scan::ScanOutput;
use crate::serialize::value_to_attribute_value;

impl<T> Serialize for ScanOutput<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ScanOutput", 5)?;
        state.serialize_field(
            "consumed_capacity",
            &self
                .consumed_capacity
                .as_ref()
                .map(crate::aws_sdk::serialize::consumed_capacity_to_value),
        )?;
        state.serialize_field("items", &self.items)?;
        state.serialize_field("count", &self.count)?;
        state.serialize_field(
            "last_evaluated_key",
            &self.last_evaluated_key.as_ref().map(|v| {
                v.iter()
                    .map(|(k, v)| (k, crate::aws_sdk::serialize::attribute_value_to_value(v)))
                    .collect::<HashMap<_, _>>()
            }),
        )?;
        state.serialize_field("scanned_count", &self.scanned_count)?;
        state.end()
    }
}

impl<'de, T> Deserialize<'de> for ScanOutput<T>
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
            Count,
            LastEvaluatedKey,
            ScannedCount,
        }

        const FIELDS: &[&str] = &[
            "consumed_capacity",
            "items",
            "count",
            "last_evaluated_key",
            "scanned_count",
        ];

        struct ScanOutputVisitor<'de, T>
        where
            T: Deserialize<'de>,
        {
            marker: std::marker::PhantomData<ScanOutput<T>>,
            lifetime: std::marker::PhantomData<&'de ()>,
        }

        impl<'de, T> Visitor<'de> for ScanOutputVisitor<'de, T>
        where
            T: Deserialize<'de>,
        {
            type Value = ScanOutput<T>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct ScanOutput")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut consumed_capacity = None;
                let mut items = None;
                let mut count = None;
                let mut last_evaluated_key = None;
                let mut scanned_count = None;

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
                        Field::Items => {
                            if items.is_some() {
                                return Err(de::Error::duplicate_field("items"));
                            }

                            items = Some(map.next_value()?);
                        }
                        Field::Count => {
                            if count.is_some() {
                                return Err(de::Error::duplicate_field("count"));
                            }

                            count = map.next_value()?;
                        }
                        Field::LastEvaluatedKey => {
                            if last_evaluated_key.is_some() {
                                return Err(de::Error::duplicate_field("last_evaluated_key"));
                            }

                            let v: Option<HashMap<String, serde_json::Value>> = map.next_value()?;

                            last_evaluated_key = if let Some(v) = v {
                                let mut map: HashMap<String, crate::AttributeValue> =
                                    HashMap::new();

                                for (k, v) in v {
                                    map.insert(
                                        k,
                                        value_to_attribute_value(v)
                                            .map_err(|err| {
                                                de::Error::custom(format!(
                                                    "Invalid value was detected as AttributeValue: {err}",
                                                ))
                                            })?,
                                    );
                                }

                                Some(map)
                            } else {
                                None
                            };
                        }
                        Field::ScannedCount => {
                            if scanned_count.is_some() {
                                return Err(de::Error::duplicate_field("scanned_count"));
                            }

                            scanned_count = map.next_value()?;
                        }
                    }
                }

                let items = items.ok_or_else(|| de::Error::missing_field("items"))?;

                Ok(ScanOutput {
                    consumed_capacity,
                    items,
                    count,
                    last_evaluated_key,
                    scanned_count,
                })
            }
        }

        deserializer.deserialize_struct(
            "ScanOutput",
            FIELDS,
            ScanOutputVisitor {
                marker: std::marker::PhantomData::<ScanOutput<T>>,
                lifetime: std::marker::PhantomData,
            },
        )
    }
}
