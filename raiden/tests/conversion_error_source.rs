use std::{collections::HashMap, error::Error};

use proptest::prelude::*;
use raiden::{
    AttributeValue, ConversionError, FromAttribute, IntoAttribute, Raiden, RaidenDocument,
    RaidenError, RaidenItem,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Validated;

impl IntoAttribute for Validated {
    fn into_attr(self) -> AttributeValue {
        "valid".to_owned().into_attr()
    }
}

impl FromAttribute for Validated {
    fn from_attr(value: Option<AttributeValue>) -> Result<Self, ConversionError> {
        if value.is_none() {
            return Err(ConversionError::ValueIsNone);
        }
        Err(ConversionError::message("custom validation failed"))
    }
}

#[derive(Raiden)]
#[raiden(table_name = "validated")]
#[allow(dead_code)]
struct RequiredItem {
    #[raiden(partition_key)]
    id: String,
    #[raiden(rename = "stored_value")]
    value: Validated,
}

#[derive(Raiden)]
#[raiden(table_name = "validated")]
#[allow(dead_code)]
struct OptionalItem {
    #[raiden(partition_key)]
    id: String,
    value: Option<Validated>,
}

#[derive(Raiden)]
#[raiden(table_name = "validated")]
#[allow(dead_code)]
struct DefaultItem {
    #[raiden(partition_key)]
    id: String,
    #[raiden(use_default)]
    value: Validated,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Detailed;

impl IntoAttribute for Detailed {
    fn into_attr(self) -> AttributeValue {
        "valid".to_owned().into_attr()
    }
}

impl FromAttribute for Detailed {
    fn from_attr(value: Option<AttributeValue>) -> Result<Self, ConversionError> {
        let value = String::from_attr(value)?;
        Err(ConversionError::message(format!("rejected: {value}")))
    }
}

#[derive(Raiden)]
#[raiden(table_name = "detailed")]
#[allow(dead_code)]
struct DetailedRequiredItem {
    #[raiden(partition_key)]
    id: String,
    #[raiden(rename = "stored_value")]
    value: Detailed,
}

#[derive(Raiden)]
#[raiden(table_name = "detailed")]
#[allow(dead_code)]
struct DetailedOptionalItem {
    #[raiden(partition_key)]
    id: String,
    value: Option<Detailed>,
}

#[derive(Raiden)]
#[raiden(table_name = "detailed")]
#[allow(dead_code)]
struct DetailedDefaultItem {
    #[raiden(partition_key)]
    id: String,
    #[raiden(use_default)]
    value: Detailed,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, RaidenDocument)]
#[serde(tag = "kind")]
enum TaggedItem {
    Valid { value: String },
}

fn item_with(value: Option<AttributeValue>) -> HashMap<String, AttributeValue> {
    let mut item = HashMap::from([("id".to_owned(), "key".to_owned().into_attr())]);
    if let Some(value) = value {
        item.insert("value".to_owned(), value);
    }
    item
}

fn assert_conversion_error(error: RaidenError, expected_attribute: &str, expected_cause: &str) {
    assert_eq!(error.source().unwrap().to_string(), expected_cause);
    assert!(error.to_string().contains(expected_cause));
    match error {
        RaidenError::AttributeConvertError { attr_name, source } => {
            assert_eq!(attr_name, expected_attribute);
            assert_eq!(source.to_string(), expected_cause);
        }
        other => panic!("expected AttributeConvertError, got {other:?}"),
    }
}

#[test]
fn required_field_preserves_custom_cause_and_renamed_attribute() {
    let mut item = item_with(None);
    item.insert("stored_value".to_owned(), "invalid".to_owned().into_attr());
    let error = RequiredItem::from_item(item).err().unwrap();
    assert_conversion_error(error, "stored_value", "custom validation failed");
}

#[test]
fn missing_field_preserves_value_is_none() {
    let error = RequiredItem::from_item(item_with(None)).err().unwrap();
    assert_conversion_error(error, "stored_value", "Value is none");
}

#[test]
fn optional_and_default_fields_preserve_custom_cause() {
    let value = Some("invalid".to_owned().into_attr());
    let optional_error = OptionalItem::from_item(item_with(value.clone()))
        .err()
        .unwrap();
    assert_conversion_error(optional_error, "value", "custom validation failed");

    let default_error = DefaultItem::from_item(item_with(value)).err().unwrap();
    assert_conversion_error(default_error, "value", "custom validation failed");
}

#[test]
fn tagged_enum_item_preserves_serde_cause() {
    let item = HashMap::from([("kind".to_owned(), "Unknown".to_owned().into_attr())]);
    let error = TaggedItem::from_item(item).err().unwrap();
    assert_eq!(
        error.source().unwrap().to_string(),
        "unknown variant `Unknown`, expected `Valid`"
    );
    match error {
        RaidenError::AttributeConvertError { attr_name, source } => {
            assert_eq!(attr_name, "TaggedItem");
            assert!(matches!(source, ConversionError::Serde(_)));
        }
        other => panic!("expected AttributeConvertError, got {other:?}"),
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn generated_custom_errors_remain_available_as_sources(
        value in "[a-zA-Z0-9 _:-]{1,64}",
        field_kind in 0u8..3,
    ) {
        let expected_cause = format!("rejected: {value}");
        let mut item = item_with(None);
        let (error, expected_attribute) = match field_kind {
            0 => {
                item.insert("stored_value".to_owned(), value.into_attr());
                (DetailedRequiredItem::from_item(item).err().unwrap(), "stored_value")
            }
            1 => {
                item.insert("value".to_owned(), value.into_attr());
                (DetailedOptionalItem::from_item(item).err().unwrap(), "value")
            }
            _ => {
                item.insert("value".to_owned(), value.into_attr());
                (DetailedDefaultItem::from_item(item).err().unwrap(), "value")
            }
        };

        let cause = error.source().unwrap().downcast_ref::<ConversionError>().unwrap();
        prop_assert!(matches!(cause, ConversionError::Serde(message) if message == &expected_cause));
        prop_assert!(error.to_string().contains(&expected_cause));
        match error {
            RaidenError::AttributeConvertError { attr_name, .. } => {
                prop_assert_eq!(attr_name, expected_attribute);
            }
            other => prop_assert!(false, "expected AttributeConvertError, got {other:?}"),
        }
    }

    #[test]
    fn generated_unknown_enum_variants_preserve_serde_errors(variant in "[A-Z][a-zA-Z0-9]{0,24}") {
        prop_assume!(variant != "Valid");
        let item = HashMap::from([("kind".to_owned(), variant.clone().into_attr())]);
        let error = TaggedItem::from_item(item).err().unwrap();
        let cause = error.source().unwrap().downcast_ref::<ConversionError>().unwrap();
        prop_assert!(matches!(cause, ConversionError::Serde(message) if message.contains(&variant)));
        match error {
            RaidenError::AttributeConvertError { attr_name, .. } => prop_assert_eq!(attr_name, "TaggedItem"),
            other => prop_assert!(false, "expected AttributeConvertError, got {other:?}"),
        }
    }
}
