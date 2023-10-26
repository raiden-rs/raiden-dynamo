use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};
use serde::ser::{Serialize, SerializeStruct, Serializer};

use crate::ops::put::PutOutput;

impl<T> Serialize for PutOutput<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("PutOutput", 2)?;
        state.serialize_field(
            "consumed_capacity",
            &self
                .consumed_capacity
                .as_ref()
                .map(|v| crate::aws_sdk::serialize::consumed_capacity_to_value(&v)),
        )?;
        state.serialize_field("item", &self.item)?;
        state.end()
    }
}

impl<'de, T> Deserialize<'de> for PutOutput<T>
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
        }

        const FIELDS: &'static [&'static str] = &["consumed_capacity", "item"];

        struct PutOutputVisitor<'de, T>
        where
            T: Deserialize<'de>,
        {
            marker: std::marker::PhantomData<PutOutput<T>>,
            lifetime: std::marker::PhantomData<&'de ()>,
        }

        impl<'de, T> Visitor<'de> for PutOutputVisitor<'de, T>
        where
            T: Deserialize<'de>,
        {
            type Value = PutOutput<T>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct PutOutput")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut consumed_capacity = None;
                let mut item = None;

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

                            item = Some(map.next_value()?);
                        }
                    }
                }

                let item = item.ok_or_else(|| de::Error::missing_field("item"))?;

                Ok(PutOutput {
                    consumed_capacity,
                    item,
                })
            }
        }

        deserializer.deserialize_struct(
            "PutOutput",
            FIELDS,
            PutOutputVisitor {
                marker: std::marker::PhantomData::<PutOutput<T>>,
                lifetime: std::marker::PhantomData,
            },
        )
    }
}
