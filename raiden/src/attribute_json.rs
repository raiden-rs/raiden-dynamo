//! Validation shared by Raiden's raw DynamoDB attribute JSON entry points.
//! Native backend AttributeValue types remain exposed; their deserializers are
//! not used unchecked at these boundaries.
use serde::de::Error as _;
use serde_json::{Error, Value};

pub(crate) fn validate_attribute(value: &Value) -> Result<(), Error> {
    let fields = value
        .as_object()
        .ok_or_else(|| Error::custom("AttributeValue must be an object"))?;
    if fields.len() != 1 {
        return Err(Error::custom(
            "AttributeValue must include exactly one type field",
        ));
    }
    let (tag, payload) = fields.iter().next().unwrap();
    match (tag.as_str(), payload) {
        ("S" | "N" | "B", Value::String(_)) | ("BOOL" | "NULL", Value::Bool(_)) => Ok(()),
        ("SS" | "NS" | "BS", Value::Array(values)) if values.iter().all(Value::is_string) => Ok(()),
        ("L", Value::Array(values)) => values.iter().try_for_each(validate_attribute),
        ("M", value @ Value::Object(_)) => validate_map(value),
        _ => Err(Error::custom(
            "Unknown AttributeValue type or invalid payload",
        )),
    }
}

pub(crate) fn validate_map(value: &Value) -> Result<(), Error> {
    value
        .as_object()
        .ok_or_else(|| Error::custom("Attribute map must be an object"))?
        .values()
        .try_for_each(validate_attribute)
}

#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
mod rusoto {
    use super::*;
    use serde::{de::DeserializeOwned, Deserialize, Deserializer};

    fn validated<'de, D, T>(
        deserializer: D,
        validate: impl FnOnce(&Value) -> Result<(), Error>,
    ) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: DeserializeOwned,
    {
        let value = Value::deserialize(deserializer)?;
        validate(&value).map_err(D::Error::custom)?;
        serde_json::from_value(value).map_err(D::Error::custom)
    }

    fn validate_request_maps(value: &Value, field: &str) -> Result<(), Error> {
        let requests = value
            .as_array()
            .ok_or_else(|| Error::custom("Requests must be an array"))?;
        for request in requests {
            validate_map(&request[field])?;
        }
        Ok(())
    }

    pub(crate) fn put_requests<'de, D>(deserializer: D) -> Result<Vec<crate::PutRequest>, D::Error>
    where
        D: Deserializer<'de>,
    {
        validated(deserializer, |v| validate_request_maps(v, "Item"))
    }

    pub(crate) fn delete_requests<'de, D>(
        deserializer: D,
    ) -> Result<Vec<crate::DeleteRequest>, D::Error>
    where
        D: Deserializer<'de>,
    {
        validated(deserializer, |v| validate_request_maps(v, "Key"))
    }

    pub(crate) fn keys_and_attributes<'de, D>(
        deserializer: D,
    ) -> Result<Option<crate::KeysAndAttributes>, D::Error>
    where
        D: Deserializer<'de>,
    {
        validated(deserializer, |v| {
            if !v.is_null() {
                let keys = v["Keys"]
                    .as_array()
                    .ok_or_else(|| Error::custom("Keys must be an array"))?;
                keys.iter().try_for_each(validate_map)?;
            }
            Ok(())
        })
    }

    pub(crate) fn item_collection_metrics<'de, D>(
        deserializer: D,
    ) -> Result<Option<crate::ItemCollectionMetrics>, D::Error>
    where
        D: Deserializer<'de>,
    {
        validated(deserializer, |v| {
            if let Some(key) = v.get("ItemCollectionKey").filter(|v| !v.is_null()) {
                validate_map(key)?;
            }
            Ok(())
        })
    }

    pub(crate) fn optional_map<'de, D>(
        deserializer: D,
    ) -> Result<Option<crate::AttributeValues>, D::Error>
    where
        D: Deserializer<'de>,
    {
        validated(deserializer, |v| {
            if !v.is_null() {
                validate_map(v)?;
            }
            Ok(())
        })
    }
}

#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
pub(crate) use rusoto::*;
