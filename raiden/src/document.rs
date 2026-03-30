use std::{
    collections::{BTreeMap, HashMap},
    ops::{Deref, DerefMut},
};

use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

use crate::{AttributeValue, ConversionError, FromAttribute, IntoAttribute};

/// Explicit wrapper for values stored as DynamoDB document attributes.
///
/// `Document<T>` is the opt-in wrapper for nested values that should be encoded
/// into DynamoDB's document model (`M`, `L`, and scalar attribute values)
/// through `serde`.
///
/// This is useful when you want a field to remain explicitly marked as a
/// document at the type level:
///
/// ```rust
/// # use raiden::Document;
/// # #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
/// # struct Profile { name: String }
/// let profile = Document::new(Profile { name: "bokuweb".to_owned() });
/// ```
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(transparent)]
pub struct Document<T>(pub T);

impl<T> Document<T> {
    /// Creates a new document wrapper around `value`.
    pub fn new(value: T) -> Self {
        Self(value)
    }

    /// Consumes the wrapper and returns the inner value.
    pub fn into_inner(self) -> T {
        self.0
    }

    /// Returns an immutable reference to the wrapped value.
    pub fn as_ref(&self) -> &T {
        &self.0
    }

    /// Returns a mutable reference to the wrapped value.
    pub fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> From<T> for Document<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> AsRef<T> for Document<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> AsMut<T> for Document<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> Deref for Document<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Document<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn conversion_error(message: impl Into<String>) -> ConversionError {
    ConversionError::Serde(message.into())
}

pub(crate) fn serialize_document<T: Serialize>(value: T) -> Result<AttributeValue, ConversionError> {
    let value = serde_json::to_value(value)
        .map_err(|err| conversion_error(format!("document serialization failed: {err}")))?;

    json_value_to_attribute_value(value)
}

#[cfg(feature = "aws-sdk")]
fn attr_null() -> AttributeValue {
    AttributeValue::Null(true)
}

#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
fn attr_null() -> AttributeValue {
    AttributeValue {
        null: Some(true),
        ..Default::default()
    }
}

#[cfg(feature = "aws-sdk")]
fn attr_bool(value: bool) -> AttributeValue {
    AttributeValue::Bool(value)
}

#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
fn attr_bool(value: bool) -> AttributeValue {
    AttributeValue {
        bool: Some(value),
        ..Default::default()
    }
}

#[cfg(feature = "aws-sdk")]
fn attr_number(value: String) -> AttributeValue {
    AttributeValue::N(value)
}

#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
fn attr_number(value: String) -> AttributeValue {
    AttributeValue {
        n: Some(value),
        ..Default::default()
    }
}

#[cfg(feature = "aws-sdk")]
fn attr_string(value: String) -> AttributeValue {
    AttributeValue::S(value)
}

#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
fn attr_string(value: String) -> AttributeValue {
    AttributeValue {
        s: Some(value),
        ..Default::default()
    }
}

#[cfg(feature = "aws-sdk")]
fn attr_list(value: Vec<AttributeValue>) -> AttributeValue {
    AttributeValue::L(value)
}

#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
fn attr_list(value: Vec<AttributeValue>) -> AttributeValue {
    AttributeValue {
        l: Some(value),
        ..Default::default()
    }
}

#[cfg(feature = "aws-sdk")]
fn attr_map(value: HashMap<String, AttributeValue>) -> AttributeValue {
    AttributeValue::M(value)
}

#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
fn attr_map(value: HashMap<String, AttributeValue>) -> AttributeValue {
    AttributeValue {
        m: Some(value),
        ..Default::default()
    }
}

fn number_string_to_json(value: String) -> Result<Value, ConversionError> {
    let parsed = serde_json::from_str::<Value>(&value)
        .map_err(|err| conversion_error(format!("invalid DynamoDB number `{value}`: {err}")))?;

    match parsed {
        Value::Number(_) => Ok(parsed),
        _ => Err(conversion_error(format!(
            "invalid DynamoDB number representation `{value}`"
        ))),
    }
}

pub(crate) fn json_value_to_attribute_value(value: Value) -> Result<AttributeValue, ConversionError> {
    match value {
        Value::Null => Ok(attr_null()),
        Value::Bool(value) => Ok(attr_bool(value)),
        Value::Number(value) => Ok(attr_number(value.to_string())),
        Value::String(value) => Ok(attr_string(value)),
        Value::Array(values) => {
            let values = values
                .into_iter()
                .map(json_value_to_attribute_value)
                .collect::<Result<Vec<_>, _>>()?;

            Ok(attr_list(values))
        }
        Value::Object(values) => {
            let values = values
                .into_iter()
                .map(|(key, value)| json_value_to_attribute_value(value).map(|value| (key, value)))
                .collect::<Result<HashMap<_, _>, _>>()?;

            Ok(attr_map(values))
        }
    }
}

#[cfg(feature = "aws-sdk")]
pub(crate) fn attribute_value_to_json_value(value: AttributeValue) -> Result<Value, ConversionError> {
    match value {
        AttributeValue::Null(_) => Ok(Value::Null),
        AttributeValue::Bool(value) => Ok(Value::Bool(value)),
        AttributeValue::N(value) => number_string_to_json(value),
        AttributeValue::S(value) => Ok(Value::String(value)),
        AttributeValue::L(values) => Ok(Value::Array(
            values
                .into_iter()
                .map(attribute_value_to_json_value)
                .collect::<Result<Vec<_>, _>>()?,
        )),
        AttributeValue::M(values) => {
            let values = values
                .into_iter()
                .map(|(key, value)| attribute_value_to_json_value(value).map(|value| (key, value)))
                .collect::<Result<serde_json::Map<_, _>, _>>()?;

            Ok(Value::Object(values))
        }
        other => Err(conversion_error(format!(
            "unsupported DynamoDB attribute for Document: {other:?}"
        ))),
    }
}

pub(crate) fn deserialize_document<T: DeserializeOwned>(
    value: Option<AttributeValue>,
) -> Result<T, ConversionError> {
    let value = value.ok_or(ConversionError::ValueIsNone)?;
    let value = attribute_value_to_json_value(value)?;
    serde_json::from_value(value).map_err(|err| ConversionError::Serde(err.to_string()))
}

#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
pub(crate) fn attribute_value_to_json_value(value: AttributeValue) -> Result<Value, ConversionError> {
    match value {
        AttributeValue {
            null: Some(true), ..
        } => Ok(Value::Null),
        AttributeValue {
            bool: Some(value), ..
        } => Ok(Value::Bool(value)),
        AttributeValue { n: Some(value), .. } => number_string_to_json(value),
        AttributeValue { s: Some(value), .. } => Ok(Value::String(value)),
        AttributeValue {
            l: Some(values), ..
        } => Ok(Value::Array(
            values
                .into_iter()
                .map(attribute_value_to_json_value)
                .collect::<Result<Vec<_>, _>>()?,
        )),
        AttributeValue {
            m: Some(values), ..
        } => {
            let values = values
                .into_iter()
                .map(|(key, value)| attribute_value_to_json_value(value).map(|value| (key, value)))
                .collect::<Result<serde_json::Map<_, _>, _>>()?;

            Ok(Value::Object(values))
        }
        other => Err(conversion_error(format!(
            "unsupported DynamoDB attribute for Document: {other:?}"
        ))),
    }
}

impl<T: IntoAttribute> IntoAttribute for HashMap<String, T> {
    fn into_attr(self) -> AttributeValue {
        let values = self
            .into_iter()
            .map(|(key, value)| (key, value.into_attr()))
            .collect::<HashMap<_, _>>();

        attr_map(values)
    }
}

impl<T: FromAttribute> FromAttribute for HashMap<String, T> {
    fn from_attr(value: Option<AttributeValue>) -> Result<Self, ConversionError> {
        #[cfg(feature = "aws-sdk")]
        let values = match value {
            Some(AttributeValue::M(values)) => values,
            _ => return Err(ConversionError::ValueIsNone),
        };

        #[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
        let values = match value {
            Some(AttributeValue { m: Some(values), .. }) => values,
            _ => return Err(ConversionError::ValueIsNone),
        };

        values
            .into_iter()
            .map(|(key, value)| T::from_attr(Some(value)).map(|value| (key, value)))
            .collect()
    }
}

impl<T: IntoAttribute> IntoAttribute for BTreeMap<String, T> {
    fn into_attr(self) -> AttributeValue {
        let values = self
            .into_iter()
            .map(|(key, value)| (key, value.into_attr()))
            .collect::<HashMap<_, _>>();

        attr_map(values)
    }
}

impl<T: FromAttribute> FromAttribute for BTreeMap<String, T> {
    fn from_attr(value: Option<AttributeValue>) -> Result<Self, ConversionError> {
        #[cfg(feature = "aws-sdk")]
        let values = match value {
            Some(AttributeValue::M(values)) => values,
            _ => return Err(ConversionError::ValueIsNone),
        };

        #[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
        let values = match value {
            Some(AttributeValue { m: Some(values), .. }) => values,
            _ => return Err(ConversionError::ValueIsNone),
        };

        values
            .into_iter()
            .map(|(key, value)| T::from_attr(Some(value)).map(|value| (key, value)))
            .collect()
    }
}

impl<T: Serialize> IntoAttribute for Document<T> {
    fn into_attr(self) -> AttributeValue {
        serialize_document(self.0)
            .expect("Document serialization failed. Use serializable document values only.")
    }
}

impl<T: DeserializeOwned> FromAttribute for Document<T> {
    fn from_attr(value: Option<AttributeValue>) -> Result<Self, ConversionError> {
        deserialize_document(value).map(Document)
    }
}

#[cfg(test)]
mod tests {
    use super::Document;
    use crate::{FromAttribute, IntoAttribute};
    use pretty_assertions::assert_eq;
    use std::collections::{BTreeMap, HashMap};

    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    struct NestedProfile {
        nickname: String,
        score: usize,
    }

    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    struct UserDocument {
        tags: Vec<String>,
        profile: NestedProfile,
        enabled: bool,
    }

    #[test]
    fn hash_map_round_trip() {
        let mut value = HashMap::new();
        value.insert("score".to_owned(), 42usize);
        value.insert("visits".to_owned(), 7usize);

        let attr = value.clone().into_attr();
        let decoded = HashMap::<String, usize>::from_attr(Some(attr)).unwrap();

        assert_eq!(decoded, value);
    }

    #[test]
    fn btree_map_round_trip() {
        let mut value = BTreeMap::new();
        value.insert("admin".to_owned(), true);
        value.insert("enabled".to_owned(), false);

        let attr = value.clone().into_attr();
        let decoded = BTreeMap::<String, bool>::from_attr(Some(attr)).unwrap();

        assert_eq!(decoded, value);
    }

    #[test]
    fn empty_map_round_trip() {
        let value = HashMap::<String, usize>::new();

        let attr = value.clone().into_attr();
        let decoded = HashMap::<String, usize>::from_attr(Some(attr)).unwrap();

        assert_eq!(decoded, value);
    }

    #[test]
    fn document_round_trip() {
        let value = Document::new(UserDocument {
            tags: vec!["rust".to_owned(), "dynamo".to_owned()],
            profile: NestedProfile {
                nickname: "bokuweb".to_owned(),
                score: 99,
            },
            enabled: true,
        });

        let attr = value.clone().into_attr();
        let decoded = Document::<UserDocument>::from_attr(Some(attr)).unwrap();

        assert_eq!(decoded, value);
    }

    #[test]
    fn map_of_documents_round_trip() {
        let mut value = HashMap::new();
        value.insert(
            "primary".to_owned(),
            Document::new(NestedProfile {
                nickname: "bokuweb".to_owned(),
                score: 100,
            }),
        );

        let attr = value.clone().into_attr();
        let decoded = HashMap::<String, Document<NestedProfile>>::from_attr(Some(attr)).unwrap();

        assert_eq!(decoded, value);
    }
}
