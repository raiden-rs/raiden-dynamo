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

pub struct Attributes(AttributeValues);

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

#[derive(Debug)]
pub enum ConversionError {
    ValueIsNone,
    ParseInt,
}

impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversionError::ValueIsNone => write!(f, "Value is none"),
            ConversionError::ParseInt => write!(f, "Parsing error of integer"),
        }
    }
}

impl std::error::Error for ConversionError {}

pub trait FromAttribute: Sized {
    fn from_attr(value: Option<AttributeValue>) -> Result<Self, ConversionError>;
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

pub fn merge_map<T>(
    map1: std::collections::HashMap<String, T>,
    map2: std::collections::HashMap<String, T>,
) -> std::collections::HashMap<String, T> {
    map1.into_iter().chain(map2).collect()
}
