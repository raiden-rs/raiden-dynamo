mod errors;
mod ops;
pub(crate) mod serialize;

use std::collections::{BTreeSet, HashMap, HashSet};

pub use self::{errors::*, ops::*};
pub use aws_sdk_dynamodb::{
    client::*,
    config::*,
    error::*,
    meta::*,
    operation::{
        batch_get_item::{builders::*, *},
        batch_write_item::{builders::*, *},
        delete_item::{builders::*, *},
        get_item::{builders::*, *},
        put_item::{builders::*, *},
        query::{builders::*, *},
        scan::{builders::*, *},
        transact_write_items::{builders::*, *},
        update_item::{builders::*, *},
    },
    primitives::*,
    types::{builders::*, *},
};

use crate::{
    AttributeType, AttributeValues, ConversionError, FromAttribute, FromStringSetItem,
    IntoAttribute, IntoStringSetItem, RaidenError,
};

impl IntoAttribute for AttributeType {
    fn into_attr(self) -> AttributeValue {
        AttributeValue::S(self.to_string())
    }
}

impl IntoAttribute for String {
    fn into_attr(self) -> AttributeValue {
        AttributeValue::S(self)
    }
}

impl FromAttribute for String {
    fn from_attr(value: Option<AttributeValue>) -> Result<Self, ConversionError> {
        match value {
            Some(v) if v.is_null() => Ok("".to_owned()),
            Some(AttributeValue::S(s)) => Ok(s),
            _ => Err(ConversionError::ValueIsNone),
        }
    }
}

impl IntoAttribute for &'_ str {
    fn into_attr(self) -> AttributeValue {
        if self.is_empty() {
            AttributeValue::Null(true)
        } else {
            AttributeValue::S(self.to_owned())
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
            AttributeValue::Null(true)
        } else {
            AttributeValue::S(s)
        }
    }
}

impl<'a> FromAttribute for std::borrow::Cow<'a, str> {
    fn from_attr(value: Option<AttributeValue>) -> Result<Self, ConversionError> {
        match value {
            Some(v) if v.is_null() => Ok(std::borrow::Cow::Owned("".to_owned())),
            Some(AttributeValue::S(s)) => Ok(std::borrow::Cow::Owned(s)),
            _ => Err(ConversionError::ValueIsNone),
        }
    }
}

macro_rules! default_attr_for_num {
    ($to: ty) => {
        impl IntoAttribute for $to {
            fn into_attr(self) -> AttributeValue {
                AttributeValue::N(format!("{self}"))
            }
        }

        impl FromAttribute for $to {
            fn from_attr(value: Option<AttributeValue>) -> Result<Self, ConversionError> {
                if let Some(AttributeValue::N(n)) = value {
                    Ok(n.parse().unwrap())
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
            AttributeValue::Null(true)
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
        AttributeValue::Bool(self)
    }
}

impl FromAttribute for bool {
    fn from_attr(value: Option<AttributeValue>) -> Result<Self, ConversionError> {
        if let Some(AttributeValue::Bool(v)) = value {
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
            AttributeValue::Null(true)
        } else {
            AttributeValue::L(self.drain(..).map(|s| s.into_attr()).collect())
        }
    }
}

impl<A: FromAttribute> FromAttribute for Vec<A> {
    fn from_attr(value: Option<AttributeValue>) -> Result<Self, ConversionError> {
        match value {
            Some(AttributeValue::L(v)) => {
                v.into_iter().map(|item| A::from_attr(Some(item))).collect()
            }
            // See. https://github.com/raiden-rs/raiden/issues/57
            Some(v) if v.is_null() => Ok(vec![]),
            None => Ok(vec![]),
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
                    AttributeValue::Ns(Default::default())
                } else {
                    AttributeValue::Ns(self.into_iter().map(|s| s.to_string()).collect())
                }
            }
        }

        impl FromAttribute for std::collections::HashSet<$to> {
            fn from_attr(value: Option<AttributeValue>) -> Result<Self, ConversionError> {
                match value {
                    Some(AttributeValue::Ns(mut nums)) => {
                        let mut results: Vec<Result<$to, ConversionError>> = nums
                            .drain(..)
                            .map(|ns| ns.parse().map_err(|_| ConversionError::ParseInt))
                            .collect();

                        results.drain(..).collect()
                    }
                    // See. https://github.com/raiden-rs/raiden/issues/57
                    Some(v) if v.is_null() => Ok(std::collections::HashSet::new()),
                    None => Ok(std::collections::HashSet::new()),
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
                    AttributeValue::Ns(Default::default())
                } else {
                    AttributeValue::Ns(self.into_iter().map(|s| s.to_string()).collect())
                }
            }
        }

        impl FromAttribute for std::collections::BTreeSet<$to> {
            fn from_attr(value: Option<AttributeValue>) -> Result<Self, ConversionError> {
                match value {
                    Some(AttributeValue::Ns(mut nums)) => {
                        let mut results: Vec<Result<$to, ConversionError>> = nums
                            .drain(..)
                            .map(|ns| ns.parse().map_err(|_| ConversionError::ParseInt))
                            .collect();

                        results.drain(..).collect()
                    }
                    // See. https://github.com/raiden-rs/raiden/issues/57
                    Some(v) if v.is_null() => Ok(std::collections::BTreeSet::new()),
                    None => Ok(std::collections::BTreeSet::new()),
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
            AttributeValue::Ss(Default::default())
        } else {
            AttributeValue::Ss(self.into_iter().map(|s| s.into_ss_item()).collect())
        }
    }
}

impl<A: std::hash::Hash + std::cmp::Eq + FromStringSetItem> FromAttribute for HashSet<A> {
    fn from_attr(value: Option<AttributeValue>) -> Result<Self, ConversionError> {
        match value {
            Some(AttributeValue::Ss(mut ss)) => ss.drain(..).map(A::from_ss_item).collect(),
            // See. https://github.com/raiden-rs/raiden/issues/57
            Some(v) if v.is_null() => Ok(HashSet::new()),
            None => Ok(HashSet::new()),
            _ => Err(ConversionError::ValueIsNone),
        }
    }
}

impl<A: std::cmp::Ord + IntoStringSetItem> IntoAttribute for BTreeSet<A> {
    fn into_attr(self) -> AttributeValue {
        if self.is_empty() {
            // See. https://github.com/raiden-rs/raiden/issues/57
            //      https://github.com/raiden-rs/raiden-dynamo/issues/64
            AttributeValue::Ss(Default::default())
        } else {
            AttributeValue::Ss(self.into_iter().map(|s| s.into_ss_item()).collect())
        }
    }
}

impl<A: std::cmp::Ord + FromStringSetItem> FromAttribute for BTreeSet<A> {
    fn from_attr(value: Option<AttributeValue>) -> Result<Self, ConversionError> {
        match value {
            Some(AttributeValue::Ss(mut ss)) => ss.drain(..).map(A::from_ss_item).collect(),
            // See. https://github.com/raiden-rs/raiden/issues/57
            Some(v) if v.is_null() => Ok(BTreeSet::new()),
            None => Ok(BTreeSet::new()),
            _ => Err(ConversionError::ValueIsNone),
        }
    }
}

pub fn is_attr_value_empty(a: &AttributeValue) -> bool {
    match &a {
        &AttributeValue::B(_)
        | &AttributeValue::Bool(_)
        | &AttributeValue::M(_)
        | &AttributeValue::N(_)
        | &AttributeValue::Null(_)
        | &AttributeValue::S(_) => false,
        &AttributeValue::Bs(v) => v.is_empty(),
        &AttributeValue::L(v) => v.is_empty(),
        &AttributeValue::Ns(v) => v.is_empty(),
        &AttributeValue::Ss(v) => v.is_empty(),
        _ => true,
    }
}

pub(crate) fn deserialize_attr_value(s: &str) -> Result<AttributeValues, RaidenError> {
    let values: HashMap<String, serde_json::Value> = match serde_json::from_str(s) {
        Ok(v) => v,
        Err(_) => return Err(RaidenError::NextTokenDecodeError),
    };

    let mut deserialized: HashMap<String, AttributeValue> = HashMap::new();

    for (k, v) in values {
        let v = crate::aws_sdk::serialize::value_to_attribute_value(v)
            .map_err(|_| RaidenError::NextTokenDecodeError)?;

        deserialized.insert(k, v);
    }

    Ok(deserialized)
}

pub(crate) fn serialize_attr_values(value: &AttributeValues) -> String {
    let m: HashMap<String, serde_json::Value> = value
        .iter()
        .map(|(k, v)| {
            (
                k.to_owned(),
                crate::aws_sdk::serialize::attribute_value_to_value(v),
            )
        })
        .collect();

    serde_json::to_string(&m).expect("should serialize")
}
