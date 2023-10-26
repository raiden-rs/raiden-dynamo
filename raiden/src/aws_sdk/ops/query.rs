use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};
use serde::ser::{Serialize, SerializeStruct, Serializer};

use crate::ops::query::QueryOutput;

impl<T> Serialize for QueryOutput<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("QueryOutput", 5)?;
        state.serialize_field(
            "consumed_capacity",
            &self
                .consumed_capacity
                .as_ref()
                .map(|v| crate::aws_sdk::serialize::consumed_capacity_to_value(&v)),
        )?;
        state.serialize_field("items", &self.items)?;
        state.serialize_field("count", &self.count)?;
        state.serialize_field("next_token", &self.next_token)?;
        state.serialize_field("scanned_count", &self.scanned_count)?;
        state.end()
    }
}

impl<'de, T> Deserialize<'de> for QueryOutput<T>
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
            NextToken,
            ScannedCount,
        }

        const FIELDS: &'static [&'static str] = &[
            "consumed_capacity",
            "items",
            "count",
            "next_token",
            "scanned_count",
        ];

        struct QueryOutputVisitor<'de, T>
        where
            T: Deserialize<'de>,
        {
            marker: std::marker::PhantomData<QueryOutput<T>>,
            lifetime: std::marker::PhantomData<&'de ()>,
        }

        impl<'de, T> Visitor<'de> for QueryOutputVisitor<'de, T>
        where
            T: Deserialize<'de>,
        {
            type Value = QueryOutput<T>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct QueryOutput")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut consumed_capacity = None;
                let mut items = None;
                let mut count = None;
                let mut next_token = None;
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
                        Field::NextToken => {
                            if next_token.is_some() {
                                return Err(de::Error::duplicate_field("next_token"));
                            }

                            next_token = map.next_value()?;
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

                Ok(QueryOutput {
                    consumed_capacity,
                    items,
                    count,
                    next_token,
                    scanned_count,
                })
            }
        }

        deserializer.deserialize_struct(
            "QueryOutput",
            FIELDS,
            QueryOutputVisitor {
                marker: std::marker::PhantomData::<QueryOutput<T>>,
                lifetime: std::marker::PhantomData,
            },
        )
    }
}
