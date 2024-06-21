use std::collections::HashMap;

use serde::de::{self, Error as _};
use serde_json::{json, Error, Map, Value};

use crate::aws_sdk::{
    serialize::{attribute_value_to_value, set_optional_value, value_to_attribute_value},
    types::DeleteRequest,
};

pub fn delete_request_to_value(v: &DeleteRequest) -> Value {
    json!({
        "key": v.key.iter().map(|(k, v)| {
            (k.clone(), attribute_value_to_value(v))
        }).collect::<HashMap<String, Value>>(),
    })
}

pub fn value_to_delete_request(value: Value) -> Result<DeleteRequest, Error> {
    if let Value::Object(m) = value {
        let mut builder = DeleteRequest::builder();

        set_optional_value!(builder, m, key, object, |m: &Map<_, _>| -> Result<_, _> {
            let mut map = HashMap::new();

            for (k, v) in m.iter() {
                let v = value_to_attribute_value(v.clone())
                    .map_err(|err| de::Error::custom(format!("{k} set in key: {err}")))?;

                map.insert(k.clone(), v);
            }

            Ok(Some(map))
        });

        builder
            .build()
            .map_err(|err| Error::custom(err.to_string()))
    } else {
        Err(de::Error::custom("value is not type of DeleteRequest"))
    }
}
