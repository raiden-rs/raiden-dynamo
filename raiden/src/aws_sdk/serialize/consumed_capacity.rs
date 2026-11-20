use std::collections::HashMap;

use serde::de;
use serde_json::{json, Error, Map, Value};

use crate::aws_sdk::{
    serialize::set_optional_value,
    types::{Capacity, ConsumedCapacity},
};

pub fn consumed_capacity_to_value(v: &ConsumedCapacity) -> Value {
    crate::aws_sdk::serialize::compact_metadata(json!({
        "TableName": v.table_name,
        "CapacityUnits": v.capacity_units,
        "ReadCapacityUnits": v.read_capacity_units,
        "WriteCapacityUnits": v.write_capacity_units,
        "Table": v.table.as_ref().map(capacity_to_value),
        "LocalSecondaryIndexes": v.local_secondary_indexes.as_ref()
            .map(|v| {
                v
                    .iter()
                    .map(|(k, v)| (k.clone(), capacity_to_value(v)))
                    .collect::<HashMap<String, Value>>()
            }),
        "GlobalSecondaryIndexes": v.global_secondary_indexes.as_ref()
            .map(|v| {
                v
                    .iter()
                    .map(|(k, v)| (k.clone(), capacity_to_value(v)))
                    .collect::<HashMap<String, Value>>()
            }),
    }))
}

pub fn value_to_consumed_capacity(value: Value) -> Result<ConsumedCapacity, Error> {
    let Value::Object(m) = value else {
        return Err(de::Error::custom("value is not type of ConsumedCapacity"));
    };

    let mut builder = ConsumedCapacity::builder();

    set_optional_value!(builder, m, table_name, String);
    set_optional_value!(builder, m, capacity_units, f64);
    set_optional_value!(builder, m, read_capacity_units, f64);
    set_optional_value!(builder, m, write_capacity_units, f64);

    if let Some(v) = crate::aws_sdk::serialize::metadata_value(&m, "table") {
        builder = builder.table(value_to_capacity(v.clone())?);
    }

    set_optional_value!(builder, m, local_secondary_indexes, object, |m: &Map<
        _,
        _,
    >|
     -> Result<
        _,
        _,
    > {
        let mut map = HashMap::new();
        for (k, v) in m.iter() {
            map.insert(k.clone(), value_to_capacity(v.clone())?);
        }

        Ok(Some(map))
    });

    set_optional_value!(builder, m, global_secondary_indexes, object, |m: &Map<
        _,
        _,
    >|
     -> Result<
        _,
        _,
    > {
        let mut map = HashMap::new();
        for (k, v) in m.iter() {
            map.insert(k.clone(), value_to_capacity(v.clone())?);
        }

        Ok(Some(map))
    });

    Ok(builder.build())
}

pub fn capacity_to_value(v: &Capacity) -> Value {
    crate::aws_sdk::serialize::compact_metadata(json!({
        "ReadCapacityUnits": v.read_capacity_units,
        "WriteCapacityUnits": v.write_capacity_units,
        "CapacityUnits": v.capacity_units,
    }))
}

pub fn value_to_capacity(value: Value) -> Result<Capacity, Error> {
    let Value::Object(m) = value else {
        return Err(de::Error::custom("value is not type of Capacity"));
    };

    let mut builder = Capacity::builder();

    set_optional_value!(builder, m, read_capacity_units, f64);
    set_optional_value!(builder, m, write_capacity_units, f64);
    set_optional_value!(builder, m, capacity_units, f64);

    Ok(builder.build())
}
