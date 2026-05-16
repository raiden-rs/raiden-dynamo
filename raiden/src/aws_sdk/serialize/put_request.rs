use std::collections::HashMap;

use serde::de::{self, Error as _};
use serde_json::{json, Error, Map, Value};

use crate::aws_sdk::{
    serialize::{attribute_value_to_value, set_optional_value, value_to_attribute_value},
    types::PutRequest,
};

pub fn put_request_to_value(v: &PutRequest) -> Value {
    json!({
        "item": v.item.iter().map(|(k, v)| {
            (k.clone(), attribute_value_to_value(v))
        }).collect::<HashMap<String, Value>>(),
    })
}

pub fn value_to_put_request(value: Value) -> Result<PutRequest, Error> {
    if let Value::Object(m) = value {
        let mut builder = PutRequest::builder();

        set_optional_value!(builder, m, item, object, |m: &Map<_, _>| -> Result<_, _> {
            let mut map = HashMap::new();

            for (k, v) in m.iter() {
                let v = value_to_attribute_value(v.clone())
                    .map_err(|err| de::Error::custom(format!("{k} set in item: {err}")))?;

                map.insert(k.clone(), v);
            }

            Ok(Some(map))
        });

        builder
            .build()
            .map_err(|err| Error::custom(err.to_string()))
    } else {
        Err(de::Error::custom("value is not type of PutRequest"))
    }
}
