use std::collections::HashMap;

use serde::de;
use serde_json::{json, Error, Map, Value};

use crate::{
    aws_sdk::serialize::{attribute_value_to_value, parse_value, set_optional_value},
    serialize::value_to_attribute_value,
};

pub fn keys_and_attributes_to_value(v: &crate::KeysAndAttributes) -> Value {
    json!({
        "keys": v.keys.as_ref().map(|v| v.iter().map(|v| v.iter().map(|(k, v)| {
            (k.clone(), attribute_value_to_value(v))
        }).collect::<HashMap<String, Value>>()).collect::<Vec<_>>()),
        "attributes_to_get": v.attributes_to_get,
        "consistent_read": v.consistent_read,
        "projection_expression": v.projection_expression,
        "expression_attribute_names": v.expression_attribute_names,
    })
}

pub fn value_to_keys_and_attributes(value: Value) -> Result<crate::KeysAndAttributes, Error> {
    if let Value::Object(m) = value {
        let mut builder = crate::KeysAndAttributes::builder();

        set_optional_value!(builder, m, keys, array, |vs: &Vec<_>| -> Result<_, _> {
            let mut values = vec![];

            for v in vs {
                let v = parse_value!(v, object, |m: &Map<_, _>| -> Result<_, _> {
                    let mut map = HashMap::new();

                    for (k, v) in m.iter() {
                        let v = value_to_attribute_value(v.clone())
                            .map_err(|err| de::Error::custom(format!("{k} set in keys: {err}")))?;

                        map.insert(k.clone(), v);
                    }

                    Ok(map)
                })?;

                values.push(v);
            }

            Ok(Some(values))
        });

        set_optional_value!(
            builder,
            m,
            attributes_to_get,
            array,
            |vs: &Vec<_>| -> Result<_, _> {
                let mut values = vec![];

                for v in vs {
                    values.push(parse_value!(v, String)?.clone());
                }

                Ok(Some(values))
            }
        );

        set_optional_value!(builder, m, consistent_read, bool);
        set_optional_value!(builder, m, projection_expression, String);

        set_optional_value!(
            builder,
            m,
            expression_attribute_names,
            object,
            |m: &Map<_, _>| -> Result<_, _> {
                let mut map = HashMap::new();

                for (k, v) in m.iter() {
                    map.insert(k.clone(), parse_value!(v, String)?.clone());
                }

                Ok(Some(map))
            }
        );

        Ok(builder.build())
    } else {
        Err(de::Error::custom("value is not type of KeysAndAttributes"))
    }
}
