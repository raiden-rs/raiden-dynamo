mod attribute_value;
mod consumed_capacity;
mod delete_request;
mod item_collection_metrics;
mod keys_and_attributes;

pub use self::{
    attribute_value::*, consumed_capacity::*, delete_request::*, item_collection_metrics::*,
    keys_and_attributes::*,
};

macro_rules! set_optional_value {
    ($builder: expr, $object: expr, $key: ident, bool) => {
        if let Some(v) = $object.get(stringify!($key)) {
            let v = crate::aws_sdk::serialize::parse_value!(v, bool).map_err(
                |err: serde_json::Error| {
                    serde::de::Error::custom(format!("{}: {err}", stringify!($key)))
                },
            )?;

            $builder = $builder.$key(*v);
        }
    };
    ($builder: expr, $object: expr, $key: ident, String) => {
        if let Some(v) = $object.get(stringify!($key)) {
            let v = crate::aws_sdk::serialize::parse_value!(v, String).map_err(
                |err: serde_json::Error| {
                    serde::de::Error::custom(format!("{}: {err}", stringify!($key)))
                },
            )?;

            $builder = $builder.$key(v);
        }
    };
    ($builder: expr, $object: expr, $key: ident, f64) => {
        if let Some(v) = $object.get(stringify!($key)) {
            let v = crate::aws_sdk::serialize::parse_value!(v, f64).map_err(
                |err: serde_json::Error| {
                    serde::de::Error::custom(format!("{}: {err}", stringify!($key)))
                },
            )?;

            $builder = $builder.$key(v);
        }
    };
    ($builder: expr, $object: expr, $key: ident, object, $closure: expr) => {
        if let Some(v) = $object.get(stringify!($key)) {
            let v = crate::aws_sdk::serialize::parse_value!(v, object, $closure).map_err(
                |err: serde_json::Error| {
                    serde::de::Error::custom(format!("{}: {err}", stringify!($key)))
                },
            )?;

            paste::paste! {
                $builder = $builder.[<set_ $key>](v);
            }
        }
    };
    ($builder: expr, $object: expr, $key: ident, array, $closure: expr) => {
        if let Some(v) = $object.get(stringify!($key)) {
            let v = crate::aws_sdk::serialize::parse_value!(v, array, $closure).map_err(
                |err: serde_json::Error| {
                    serde::de::Error::custom(format!("{}: {err}", stringify!($key)))
                },
            )?;

            paste::paste! {
                $builder = $builder.[<set_ $key>](v);
            }
        }
    };
}

macro_rules! parse_value {
    ($value: expr, bool) => {{
        if let serde_json::Value::Bool(v) = $value {
            Ok(v)
        } else {
            Err(serde::de::Error::custom("value must be type of bool"))
        }
    }};
    ($value: expr, String) => {{
        if let serde_json::Value::String(v) = $value {
            Ok(v)
        } else {
            Err(serde::de::Error::custom("value must be type of String"))
        }
    }};
    ($value: expr, f64) => {{
        if let serde_json::Value::Number(v) = $value {
            v.as_f64()
                .ok_or_else(|| de::Error::custom("value must be type of f64"))
        } else {
            Err(de::Error::custom("value must be type of Number"))
        }
    }};
    ($value: expr, object, $closure: expr) => {{
        if let serde_json::Value::Object(__m) = $value {
            Ok($closure(__m)?)
        } else {
            Err(serde::de::Error::custom("value must be type of Object"))
        }
    }};
    ($value: expr, array, $closure: expr) => {{
        if let serde_json::Value::Array(__vs) = $value {
            Ok($closure(__vs)?)
        } else {
            Err(serde::de::Error::custom("value must be type of Array"))
        }
    }};
}

use {parse_value, set_optional_value};
