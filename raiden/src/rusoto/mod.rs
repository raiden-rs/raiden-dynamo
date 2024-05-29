mod errors;
mod ops;

#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
pub use rusoto_dynamodb::*;

#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
pub use rusoto_core::*;

pub use self::ops::*;
pub use rusoto_credential::*;

use std::collections::{BTreeSet, HashSet};

use crate::{
    AttributeType, AttributeValues, ConversionError, FromAttribute, FromStringSetItem,
    IntoAttribute, IntoStringSetItem, RaidenError,
};

impl IntoAttribute for AttributeType {
    fn into_attr(self) -> AttributeValue {
        AttributeValue {
            s: Some(self.to_string()),
            ..AttributeValue::default()
        }
    }
}

impl IntoAttribute for String {
    fn into_attr(self) -> AttributeValue {
        // Empty String is allowed since 2020/5
        // https://aws.amazon.com/jp/about-aws/whats-new/2020/05/amazon-dynamodb-now-supports-empty-values-for-non-key-string-and-binary-attributes-in-dynamodb-tables/
        AttributeValue {
            s: Some(self),
            ..AttributeValue::default()
        }
    }
}

impl FromAttribute for String {
    fn from_attr(value: Option<AttributeValue>) -> Result<Self, ConversionError> {
        match value {
            // See. https://github.com/raiden-rs/raiden/issues/58
            Some(AttributeValue { null: Some(v), .. }) if v => Ok("".to_owned()),
            Some(AttributeValue { s: Some(v), .. }) => Ok(v),
            _ => Err(ConversionError::ValueIsNone),
        }
    }
}

impl IntoAttribute for &'_ str {
    fn into_attr(self) -> AttributeValue {
        if self.is_empty() {
            // See. https://github.com/raiden-rs/raiden-dynamo/issues/58
            AttributeValue {
                null: Some(true),
                ..Default::default()
            }
        } else {
            AttributeValue {
                s: Some(self.to_owned()),
                ..AttributeValue::default()
            }
        }
    }
}

impl<'a> IntoAttribute for std::borrow::Cow<'a, str> {
    fn into_attr(self) -> AttributeValue {
        let s = match self {
            std::borrow::Cow::Owned(o) => o,
            std::borrow::Cow::Borrowed(b) => b.to_owned(),
        };

        if s.is_empty() {
            // See. https://github.com/raiden-rs/raiden-dynamo/issues/58
            AttributeValue {
                null: Some(true),
                ..Default::default()
            }
        } else {
            AttributeValue {
                s: Some(s),
                ..AttributeValue::default()
            }
        }
    }
}

impl<'a> FromAttribute for std::borrow::Cow<'a, str> {
    fn from_attr(value: Option<AttributeValue>) -> Result<Self, ConversionError> {
        match value {
            // See. https://github.com/raiden-rs/raiden/issues/58
            Some(AttributeValue { null: Some(v), .. }) if v => {
                Ok(std::borrow::Cow::Owned("".to_owned()))
            }
            Some(AttributeValue { s: Some(v), .. }) => Ok(std::borrow::Cow::Owned(v)),
            _ => Err(ConversionError::ValueIsNone),
        }
    }
}

macro_rules! default_attr_for_num {
    ($to: ty) => {
        impl IntoAttribute for $to {
            fn into_attr(self) -> AttributeValue {
                AttributeValue {
                    n: Some(format!("{self}")),
                    ..AttributeValue::default()
                }
            }
        }

        impl FromAttribute for $to {
            fn from_attr(value: Option<AttributeValue>) -> Result<Self, ConversionError> {
                if let Some(AttributeValue { n: Some(v), .. }) = value {
                    Ok(v.parse().unwrap())
                } else {
                    Err(ConversionError::ValueIsNone)
                }
            }
        }
    };
}

pub(crate) use default_attr_for_num;

impl<T: IntoAttribute> IntoAttribute for Option<T> {
    fn into_attr(self) -> AttributeValue {
        if let Some(value) = self {
            value.into_attr()
        } else {
            AttributeValue {
                null: Some(true),
                ..Default::default()
            }
        }
    }
}

impl<T: FromAttribute> FromAttribute for Option<T> {
    fn from_attr(value: Option<AttributeValue>) -> Result<Self, ConversionError> {
        Ok(if let Some(v) = value {
            Some(FromAttribute::from_attr(Some(v))?)
        } else {
            None
        })
    }
}

impl IntoAttribute for bool {
    fn into_attr(self) -> AttributeValue {
        AttributeValue {
            bool: Some(self),
            ..AttributeValue::default()
        }
    }
}

impl FromAttribute for bool {
    fn from_attr(value: Option<AttributeValue>) -> Result<Self, ConversionError> {
        if let Some(AttributeValue { bool: Some(v), .. }) = value {
            Ok(v)
        } else {
            Err(ConversionError::ValueIsNone)
        }
    }
}

impl<A: IntoAttribute> IntoAttribute for Vec<A> {
    fn into_attr(mut self) -> AttributeValue {
        if self.is_empty() {
            // See. https://github.com/raiden-rs/raiden/issues/57
            AttributeValue {
                null: Some(true),
                ..Default::default()
            }
        } else {
            AttributeValue {
                l: Some(self.drain(..).map(|s| s.into_attr()).collect()),
                ..AttributeValue::default()
            }
        }
    }
}

impl<A: FromAttribute> FromAttribute for Vec<A> {
    fn from_attr(value: Option<AttributeValue>) -> Result<Self, ConversionError> {
        match value {
            Some(AttributeValue { l: Some(v), .. }) => {
                v.into_iter().map(|item| A::from_attr(Some(item))).collect()
            }
            // See. https://github.com/raiden-rs/raiden/issues/57
            Some(AttributeValue {
                null: Some(true), ..
            })
            | None => Ok(vec![]),
            _ => Err(ConversionError::ValueIsNone),
        }
    }
}

macro_rules! default_number_hash_set_convertor {
    ($to: ty) => {
        impl IntoAttribute for std::collections::HashSet<$to> {
            fn into_attr(self) -> AttributeValue {
                if self.is_empty() {
                    // See. https://github.com/raiden-rs/raiden/issues/57
                    //      https://github.com/raiden-rs/raiden-dynamo/issues/64
                    AttributeValue::default()
                } else {
                    AttributeValue {
                        ns: Some(self.into_iter().map(|s| s.to_string()).collect()),
                        ..AttributeValue::default()
                    }
                }
            }
        }

        impl FromAttribute for std::collections::HashSet<$to> {
            fn from_attr(value: Option<AttributeValue>) -> Result<Self, ConversionError> {
                match value {
                    Some(AttributeValue {
                        ns: Some(mut nums), ..
                    }) => {
                        let mut results: Vec<Result<$to, ConversionError>> = nums
                            .drain(..)
                            .map(|ns| ns.parse().map_err(|_| ConversionError::ParseInt))
                            .collect();

                        results.drain(..).collect()
                    }
                    // See. https://github.com/raiden-rs/raiden/issues/57
                    Some(AttributeValue {
                        null: Some(true), ..
                    })
                    | None => Ok(std::collections::HashSet::new()),
                    _ => Err(ConversionError::ValueIsNone),
                }
            }
        }
    };
}

pub(crate) use default_number_hash_set_convertor;

macro_rules! default_number_btree_set_convertor {
    ($to: ty) => {
        impl IntoAttribute for std::collections::BTreeSet<$to> {
            fn into_attr(self) -> AttributeValue {
                if self.is_empty() {
                    // See. https://github.com/raiden-rs/raiden/issues/57
                    //      https://github.com/raiden-rs/raiden-dynamo/issues/64
                    AttributeValue::default()
                } else {
                    AttributeValue {
                        ns: Some(self.into_iter().map(|s| s.to_string()).collect()),
                        ..AttributeValue::default()
                    }
                }
            }
        }

        impl FromAttribute for std::collections::BTreeSet<$to> {
            fn from_attr(value: Option<AttributeValue>) -> Result<Self, ConversionError> {
                match value {
                    Some(AttributeValue {
                        ns: Some(mut nums), ..
                    }) => {
                        let mut results: Vec<Result<$to, ConversionError>> = nums
                            .drain(..)
                            .map(|ns| ns.parse().map_err(|_| ConversionError::ParseInt))
                            .collect();

                        results.drain(..).collect()
                    }
                    // See. https://github.com/raiden-rs/raiden/issues/57
                    Some(AttributeValue {
                        null: Some(true), ..
                    })
                    | None => Ok(std::collections::BTreeSet::new()),
                    _ => Err(ConversionError::ValueIsNone),
                }
            }
        }
    };
}

pub(crate) use default_number_btree_set_convertor;

impl<A: std::hash::Hash + IntoStringSetItem> IntoAttribute for HashSet<A> {
    fn into_attr(self) -> AttributeValue {
        if self.is_empty() {
            // See. https://github.com/raiden-rs/raiden/issues/57
            //      https://github.com/raiden-rs/raiden-dynamo/issues/64
            AttributeValue::default()
        } else {
            AttributeValue {
                ss: Some(self.into_iter().map(|s| s.into_ss_item()).collect()),
                ..AttributeValue::default()
            }
        }
    }
}

impl<A: std::hash::Hash + std::cmp::Eq + FromStringSetItem> FromAttribute for HashSet<A> {
    fn from_attr(value: Option<AttributeValue>) -> Result<Self, ConversionError> {
        match value {
            Some(AttributeValue {
                ss: Some(mut ss), ..
            }) => ss.drain(..).map(A::from_ss_item).collect(),
            // See. https://github.com/raiden-rs/raiden/issues/57
            Some(AttributeValue {
                null: Some(true), ..
            })
            | None => Ok(HashSet::new()),
            _ => Err(ConversionError::ValueIsNone),
        }
    }
}

impl<A: std::cmp::Ord + IntoStringSetItem> IntoAttribute for BTreeSet<A> {
    fn into_attr(self) -> AttributeValue {
        if self.is_empty() {
            // See. https://github.com/raiden-rs/raiden/issues/57
            //      https://github.com/raiden-rs/raiden-dynamo/issues/64
            AttributeValue::default()
        } else {
            AttributeValue {
                ss: Some(self.into_iter().map(|s| s.into_ss_item()).collect()),
                ..AttributeValue::default()
            }
        }
    }
}

impl<A: std::cmp::Ord + FromStringSetItem> FromAttribute for BTreeSet<A> {
    fn from_attr(value: Option<AttributeValue>) -> Result<Self, ConversionError> {
        match value {
            Some(AttributeValue {
                ss: Some(mut ss), ..
            }) => ss.drain(..).map(A::from_ss_item).collect(),
            // See. https://github.com/raiden-rs/raiden/issues/57
            Some(AttributeValue {
                null: Some(true), ..
            })
            | None => Ok(BTreeSet::new()),
            _ => Err(ConversionError::ValueIsNone),
        }
    }
}

pub fn is_attr_value_empty(a: &AttributeValue) -> bool {
    a == &AttributeValue::default()
}

pub(crate) fn deserialize_attr_value(s: &str) -> Result<AttributeValues, RaidenError> {
    match serde_json::from_str(s) {
        Ok(deserialized) => Ok(deserialized),
        Err(_) => Err(super::RaidenError::NextTokenDecodeError),
    }
}

pub(crate) fn serialize_attr_values(value: &AttributeValues) -> String {
    serde_json::to_string(value).expect("should serialize")
}
