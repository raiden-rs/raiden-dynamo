use std::collections::HashMap;

use serde::de;
use serde_json::{json, Error, Map, Value};

use crate::aws_sdk::{
    serialize::{
        attribute_value_to_value, parse_value, set_optional_value, value_to_attribute_value,
    },
    types::ItemCollectionMetrics,
};

pub fn item_collection_metrics_to_value(v: &ItemCollectionMetrics) -> Value {
    json!({
        "item_collection_key": v.item_collection_key.as_ref().map(|v| {
            v
                .iter()
                .map(|(k, v)| (k.clone(), attribute_value_to_value(v)))
                .collect::<HashMap<String, Value>>()
        }),
        "size_estimate_range_gb": v.size_estimate_range_gb,
    })
}

pub fn value_to_item_collection_metrics(value: Value) -> Result<ItemCollectionMetrics, Error> {
    if let Value::Object(m) = value {
        let mut builder = ItemCollectionMetrics::builder();

        set_optional_value!(
            builder,
            m,
            item_collection_key,
            object,
            |m: &Map<_, _>| -> Result<_, _> {
                let mut map = HashMap::new();

                for (k, v) in m.iter() {
                    let v = value_to_attribute_value(v.clone()).map_err(|err| {
                        de::Error::custom(format!("{k} set in item_collection_key: {err}"))
                    })?;

                    map.insert(k.clone(), v);
                }

                Ok(Some(map))
            }
        );

        set_optional_value!(
            builder,
            m,
            size_estimate_range_gb,
            array,
            |vs: &Vec<_>| -> Result<_, _> {
                let mut values = vec![];

                for v in vs {
                    values.push(parse_value!(v, f64)?);
                }

                Ok(Some(values))
            }
        );

        Ok(builder.build())
    } else {
        Err(de::Error::custom(
            "value is not type of ItemCollectionMetrics",
        ))
    }
}
