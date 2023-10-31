use std::collections::HashMap;

use aws_sdk_dynamodb::primitives::Blob;
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::de::{self, Error as _};
use serde_json::{json, Error, Value};

use crate::AttributeValue;

pub fn attribute_value_to_value(value: &AttributeValue) -> Value {
    match value {
        AttributeValue::B(v) => json!({ "B": STANDARD.encode(v) }),
        AttributeValue::Bool(v) => json!({ "BOOL": v }),
        AttributeValue::Bs(vs) => json!({ "BS": [
            vs.iter().map(|v| json!({ "B": STANDARD.encode(v) })).collect::<Vec<_>>(),
        ]}),
        AttributeValue::L(vs) => json!({ "L": [
            vs.iter().map(attribute_value_to_value).collect::<Vec<_>>(),
        ]}),
        AttributeValue::M(vs) => json!({ "M": vs.iter().map(|(k, v)| {
            (k.clone(), attribute_value_to_value(v))
        }).collect::<std::collections::HashMap<String, Value>>() }),
        AttributeValue::N(v) => json!({ "N": v }),
        AttributeValue::Ns(vs) => json!({ "NS": [
            vs.iter().map(|v| json!({ "N": v })).collect::<Vec<_>>(),
        ]}),
        AttributeValue::Null(v) => json!({ "NULL": v }),
        AttributeValue::S(v) => json!({ "S": v }),
        AttributeValue::Ss(vs) => json!({ "SS": [
            vs.iter().map(|v| json!({ "S": v })).collect::<Vec<_>>(),
        ]}),
        _ => {
            panic!("Unknown data type. Consider upgrading your SDK to the latest version.")
        }
    }
}

pub fn value_to_attribute_value(value: Value) -> Result<AttributeValue, Error> {
    let Value::Object(value) = value else {
        return Err(de::Error::custom("value is not type of AttributeValue"));
    };

    if value.len() != 1 {
        return Err(de::Error::custom(
            "AttributeValue must include only 1 field",
        ));
    }

    let (ty, value) = value.into_iter().next().unwrap();
    let v = match (ty.as_str(), value) {
        ("B", Value::String(s)) => {
            let b = STANDARD.decode(s).map_err(Error::custom)?;

            AttributeValue::B(Blob::new(b))
        }
        ("BOOL", Value::Bool(b)) => AttributeValue::Bool(b),
        ("BS", Value::Array(vs)) => {
            let mut values = vec![];

            for (i, v) in vs.into_iter().enumerate() {
                match v {
                    Value::String(s) => {
                        let b = STANDARD.decode(s).map_err(Error::custom)?;

                        values.push(Blob::new(b))
                    }
                    _ => {
                        return Err(Error::custom(format!(
                            "Unexpected value was detected in BS field at index of {i}",
                        )))
                    }
                };
            }

            AttributeValue::Bs(values)
        }
        ("L", Value::Array(vs)) => {
            let mut values = vec![];

            for (i, v) in vs.into_iter().enumerate() {
                let v = value_to_attribute_value(v).map_err(|err| {
                    Error::custom(format!(
                        "Unexpected value was detected in L field at index of {i}: {err}",
                    ))
                })?;

                values.push(v);
            }

            AttributeValue::L(values)
        }
        ("M", Value::Object(m)) => {
            let mut values: HashMap<String, AttributeValue> = HashMap::new();

            for (k, v) in m {
                let v = value_to_attribute_value(v).map_err(|err| {
                    Error::custom(format!(
                        "Unexpected value was detected in M field at key of {k}: {err}",
                    ))
                })?;

                values.insert(k, v);
            }

            AttributeValue::M(values)
        }
        ("N", Value::String(s)) => AttributeValue::N(s),
        ("NS", Value::Array(vs)) => {
            let mut values = vec![];

            for (i, v) in vs.into_iter().enumerate() {
                match v {
                    Value::String(s) => {
                        values.push(s);
                    }
                    _ => {
                        return Err(Error::custom(format!(
                            "Unexpected value was detected in NS field at index of {i}",
                        )))
                    }
                };
            }

            AttributeValue::Ns(values)
        }
        ("NULL", Value::Null) => AttributeValue::Null(true),
        ("S", Value::String(s)) => AttributeValue::S(s),
        ("SS", Value::Array(vs)) => {
            let mut values = vec![];

            for (i, v) in vs.into_iter().enumerate() {
                match v {
                    Value::String(s) => {
                        values.push(s);
                    }
                    _ => {
                        return Err(Error::custom(format!(
                            "Unexpected value was detected in SS field at index of {i}",
                        )))
                    }
                };
            }

            AttributeValue::Ss(values)
        }
        _ => return Err(Error::custom("Unexpected data type was detected")),
    };

    Ok(v)
}
