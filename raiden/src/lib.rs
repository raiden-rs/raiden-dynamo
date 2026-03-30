#[cfg(all(feature = "rusoto", feature = "rusoto_rustls"))]
compile_error!("feature \"rusoto\" and \"rusoto_rustls\" cannot be enabled at the same time.");

#[cfg(any(
    all(feature = "aws-sdk", feature = "rusoto"),
    all(feature = "aws-sdk", feature = "rusoto_rustls")
))]
compile_error!(
    "feature \"aws-sdk\" and \"rusoto\" or \"rusoto_rustls\" cannot be enabled at the same time."
);

#[macro_use]
extern crate serde_derive;

pub mod condition;
mod document;
pub mod errors;
pub mod filter_expression;
pub mod id_generator;
pub mod key_condition;
pub mod next_token;
pub mod ops;
pub mod retry;
pub mod types;
pub mod update_expression;
pub mod value_id;

#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
mod rusoto;

#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
pub use self::rusoto::*;

#[cfg(feature = "aws-sdk")]
pub mod aws_sdk;

#[cfg(feature = "aws-sdk")]
pub use self::aws_sdk::{types::AttributeValue, *};

pub use condition::*;
pub use document::*;
pub use errors::*;
pub use filter_expression::*;
pub use key_condition::*;
pub use next_token::*;
pub use ops::*;
pub use retry::*;

pub use id_generator::*;
pub use raiden_derive::*;
pub use types::*;
pub use value_id::*;

pub use safe_builder::Builder;

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
pub enum AttributeType {
    S,    // String
    SS,   // String Set
    N,    // Number
    NS,   // Number Set
    B,    // Binary
    BS,   // Binary Set
    BOOL, // Boolean
    NULL, // Null
    L,    // List
    M,    // Map
}

impl std::fmt::Display for AttributeType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

pub type AttributeNames = std::collections::HashMap<String, String>;
pub type AttributeValues = std::collections::HashMap<String, AttributeValue>;

pub trait IntoAttrName: Sized + Copy {
    fn into_attr_name(self) -> String;
}

pub trait ToAttrNames: Sized {
    fn to_attr_names(&self) -> AttributeNames;
}

pub trait IntoAttrValues: Sized {
    fn into_attr_values(self) -> AttributeValues;
}

pub trait IntoStringSetItem: Sized {
    fn into_ss_item(self) -> String;
}

pub trait ToAttrMaps: Sized {
    fn to_attr_maps(&self) -> (AttributeNames, AttributeValues);
}

pub trait IntoAttribute: Sized {
    fn into_attr(self) -> AttributeValue;
}

/// Marker trait for types that should be serialized as DynamoDB documents.
///
/// This trait is typically implemented via `#[derive(RaidenDocument)]`.
/// Its default method uses `serde` to encode the value into DynamoDB's
/// document model.
pub trait IntoDocumentAttr: serde::Serialize + Sized {
    /// Serializes `self` into a DynamoDB attribute value.
    fn into_document_attr(self) -> Result<AttributeValue, ConversionError> {
        crate::document::serialize_document(self)
    }
}

#[derive(Debug)]
pub enum ConversionError {
    ValueIsNone,
    ParseInt,
    Serde(String),
}

impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversionError::ValueIsNone => write!(f, "Value is none"),
            ConversionError::ParseInt => write!(f, "Parsing error of integer"),
            ConversionError::Serde(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for ConversionError {}

pub trait FromAttribute: Sized {
    fn from_attr(value: Option<AttributeValue>) -> Result<Self, ConversionError>;
}

/// Marker trait for types that should be decoded from DynamoDB documents.
///
/// This trait is typically implemented via `#[derive(RaidenDocument)]`.
/// Its default method uses `serde` to rebuild the Rust value from a DynamoDB
/// document attribute.
pub trait FromDocumentAttr: serde::de::DeserializeOwned + Sized {
    /// Deserializes a DynamoDB attribute value into `Self`.
    fn from_document_attr(value: Option<AttributeValue>) -> Result<Self, ConversionError> {
        crate::document::deserialize_document(value)
    }
}

impl<T: FromAttribute> ResolveAttribute for T {}

pub trait FromStringSetItem: Sized {
    fn from_ss_item(value: String) -> Result<Self, ConversionError>;
}

default_attr_for_num!(usize);
default_attr_for_num!(u64);
default_attr_for_num!(u32);
default_attr_for_num!(u16);
default_attr_for_num!(u8);

default_attr_for_num!(isize);
default_attr_for_num!(i64);
default_attr_for_num!(i32);
default_attr_for_num!(i16);
default_attr_for_num!(i8);

default_attr_for_num!(f32);
default_attr_for_num!(f64);

impl IntoStringSetItem for String {
    fn into_ss_item(self) -> String {
        self
    }
}

impl FromStringSetItem for String {
    fn from_ss_item(value: String) -> Result<Self, ConversionError> {
        Ok(value)
    }
}

default_number_hash_set_convertor!(usize);
default_number_hash_set_convertor!(u64);
default_number_hash_set_convertor!(u32);
default_number_hash_set_convertor!(u16);
default_number_hash_set_convertor!(u8);

default_number_hash_set_convertor!(isize);
default_number_hash_set_convertor!(i64);
default_number_hash_set_convertor!(i32);
default_number_hash_set_convertor!(i16);
default_number_hash_set_convertor!(i8);

default_number_btree_set_convertor!(usize);
default_number_btree_set_convertor!(u64);
default_number_btree_set_convertor!(u32);
default_number_btree_set_convertor!(u16);
default_number_btree_set_convertor!(u8);

default_number_btree_set_convertor!(isize);
default_number_btree_set_convertor!(i64);
default_number_btree_set_convertor!(i32);
default_number_btree_set_convertor!(i16);
default_number_btree_set_convertor!(i8);

pub trait ResolveAttribute: Sized + FromAttribute {
    fn resolve_attr(
        key: &str,
        map: &mut std::collections::HashMap<String, AttributeValue>,
    ) -> Option<AttributeValue> {
        map.remove(key)
    }
}

/// Describes a type that can be projected from a DynamoDB item.
///
/// Implementations are generated by `#[derive(Raiden)]` for source models and
/// by `#[derive(RaidenIndex)]` for custom index projection models.
///
/// When a source model uses `#[raiden(omit_gsi = "...")]`, `#[derive(Raiden)]`
/// also generates a default GSI projection item for that index.
#[allow(clippy::result_large_err)]
pub trait RaidenItem: Sized {
    /// Returns the expression attribute names required to read this type.
    fn attribute_names() -> Option<AttributeNames>;

    /// Returns the projection expression required to read this type.
    fn projection_expression() -> Option<String>;

    /// Decodes the type from a raw DynamoDB attribute map.
    fn from_item(item: AttributeValues) -> Result<Self, RaidenError>;
}

/// Marks a projected item type as belonging to a specific source model and GSI.
///
/// This trait is generated by `#[derive(RaidenIndex)]` and by the automatic
/// `omit_gsi`-based projection generation in `#[derive(Raiden)]`.
///
/// Typed GSI builders use this trait to switch both the projection expression
/// and the output item type.
pub trait RaidenIndexItem<Source>: RaidenItem {
    /// The DynamoDB GSI name associated with this projection type.
    const GSI_NAME: &'static str;
}

pub fn merge_map<T>(
    map1: std::collections::HashMap<String, T>,
    map2: std::collections::HashMap<String, T>,
) -> std::collections::HashMap<String, T> {
    map1.into_iter().chain(map2).collect()
}
