//! Shared fixtures make the two backends obey the same attribute contract.
use raiden::*;
use serde_json::json;
use std::borrow::Cow;
use std::collections::{BTreeSet, HashSet};

fn null(value: bool) -> AttributeValue {
    #[cfg(feature = "aws-sdk")]
    return AttributeValue::Null(value);
    #[cfg(feature = "rusoto")]
    return AttributeValue {
        null: Some(value),
        ..Default::default()
    };
}

#[test]
fn null_flag_must_be_true_to_read_as_null() {
    assert!(is_null_attribute_value(&null(true)));
    assert!(!is_null_attribute_value(&null(false)));
    macro_rules! check {
        ($($ty:ty),* $(,)?) => {$(
            assert!(<$ty>::from_attr(Some(null(true))).is_ok(), stringify!($ty));
            assert!(<$ty>::from_attr(Some(null(false))).is_err(), stringify!($ty));
        )*};
    }
    check!(
        String,
        Cow<'static, str>,
        Option<String>,
        Vec<String>,
        HashSet<String>,
        BTreeSet<String>,
        HashSet<usize>,
        BTreeSet<usize>
    );
    assert!(String::from_attr(None).is_err());
    assert_eq!(Option::<String>::from_attr(None).unwrap(), None);
    assert!(Vec::<String>::from_attr(None).unwrap().is_empty());
    assert!(HashSet::<String>::from_attr(None).unwrap().is_empty());
    assert!(BTreeSet::<usize>::from_attr(None).unwrap().is_empty());
}

#[test]
fn batch_put_output_preserves_all_attribute_json_variants() {
    // This is persisted public output, rather than the HTTP request encoder.
    let attributes = json!({
        "binary": {"B": "AP8="}, "binary_set": {"BS": ["AP8=", "eA=="]},
        "bool": {"BOOL": false}, "number": {"N": "0"},
        "numbers": {"NS": ["1", "2"]}, "string": {"S": ""},
        "strings": {"SS": ["a", "b"]},
        "list": {"L": [{"S": "x"}, {"M": {"child": {"L": []}}}]},
        "empty_list": {"L": []}, "empty_map": {"M": {}},
        "empty_strings": {"SS": []}, "empty_numbers": {"NS": []},
        "empty_binaries": {"BS": []}
    });
    let fixture = json!({"consumed_capacity": null, "unprocessed_items": [{"Item": attributes}]});
    let output: raiden::ops::batch_put::BatchPutOutput =
        serde_json::from_value(fixture.clone()).unwrap();
    assert_eq!(serde_json::to_value(&output).unwrap(), fixture);
    let decoded: raiden::ops::batch_put::BatchPutOutput =
        serde_json::from_value(serde_json::to_value(&output).unwrap()).unwrap();
    assert_eq!(decoded, output);
}

#[test]
fn batch_put_output_preserves_both_null_flags() {
    for flag in [true, false] {
        let fixture = json!({"consumed_capacity": null,
            "unprocessed_items": [{"Item": {"null": {"NULL": flag}}}]});
        let output: raiden::ops::batch_put::BatchPutOutput =
            serde_json::from_value(fixture.clone()).unwrap();
        assert_eq!(serde_json::to_value(output).unwrap(), fixture);
    }
}

#[test]
fn batch_put_capacity_metadata_round_trips_with_optional_null_fields() {
    let fixture = json!({"consumed_capacity": [{
        "TableName": "items", "CapacityUnits": 1.0,
        "ReadCapacityUnits": null, "WriteCapacityUnits": null,
        "Table": {"CapacityUnits": 1.0, "ReadCapacityUnits": null, "WriteCapacityUnits": null},
        "LocalSecondaryIndexes": null, "GlobalSecondaryIndexes": null
    }], "unprocessed_items": []});
    let output: raiden::ops::batch_put::BatchPutOutput =
        serde_json::from_value(fixture.clone()).unwrap();
    assert_eq!(
        serde_json::to_value(output).unwrap(),
        json!({
        "consumed_capacity": [{"TableName": "items", "CapacityUnits": 1.0,
            "Table": {"CapacityUnits": 1.0}}], "unprocessed_items": []})
    );
    let fixture = json!({"consumed_capacity": [{
        "TableName": null, "CapacityUnits": null,
        "ReadCapacityUnits": null, "WriteCapacityUnits": null,
        "Table": null, "LocalSecondaryIndexes": null, "GlobalSecondaryIndexes": null
    }], "unprocessed_items": []});
    let output: raiden::ops::batch_put::BatchPutOutput =
        serde_json::from_value(fixture.clone()).unwrap();
    assert_eq!(
        serde_json::to_value(output).unwrap(),
        json!({"consumed_capacity": [{}], "unprocessed_items": []})
    );
}

#[test]
fn existing_empty_scalar_and_option_conversions_are_unchanged() {
    let owned = String::new().into_attr();
    assert!(!is_null_attribute_value(&owned));
    assert_eq!(String::from_attr(Some(owned)).unwrap(), "");
    assert!(is_null_attribute_value(&"".into_attr()));
    assert!(is_null_attribute_value(&Cow::Borrowed("").into_attr()));
    assert!(is_null_attribute_value(&Vec::<String>::new().into_attr()));
    assert!(is_null_attribute_value(&None::<String>.into_attr()));
    assert_eq!(Option::<String>::from_attr(Some(null(true))).unwrap(), None);
}

#[test]
fn nested_output_metadata_matches_rusoto_json() {
    let keys = json!({"consumed_capacity": null, "items": [], "unprocessed_keys": {
        "Keys": [{"id": {"S": "a"}}, {"id": {"N": "1"}}, {"id": {"B": "AP8="}}],
        "AttributesToGet": ["id"], "ConsistentRead": false,
        "ProjectionExpression": "#id", "ExpressionAttributeNames": {"#id": "id"}
    }});
    let decoded: raiden::ops::batch_get::BatchGetOutput<serde_json::Value> =
        serde_json::from_value(keys.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), keys);
    let delete = json!({"consumed_capacity": null,
        "unprocessed_items": [{"Key": {"id": {"S": "a"}}}]});
    let decoded: raiden::ops::batch_delete::BatchDeleteOutput =
        serde_json::from_value(delete.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), delete);
    for metrics in [
        json!({}),
        json!({"ItemCollectionKey": {"id": {"S": "a"}},
        "SizeEstimateRangeGB": [0.0, 1.0]}),
    ] {
        let update = json!({"consumed_capacity": null, "item": null,
            "item_collection_metrics": metrics});
        let decoded: raiden::ops::update::UpdateOutput<serde_json::Value> =
            serde_json::from_value(update.clone()).unwrap();
        assert_eq!(serde_json::to_value(decoded).unwrap(), update);
    }
}

#[cfg(feature = "aws-sdk")]
#[test]
fn legacy_sdk_metadata_is_still_readable() {
    let old = json!({"consumed_capacity": [{"table_name": "items", "capacity_units": 1.0,
        "table": {"capacity_units": 1.0}, "local_secondary_indexes": null}],
        "unprocessed_items": [{"item": {"id": {"S": "a"}}}]});
    let decoded: raiden::ops::batch_put::BatchPutOutput = serde_json::from_value(old).unwrap();
    assert_eq!(
        serde_json::to_value(decoded).unwrap(),
        json!({
        "consumed_capacity": [{"TableName": "items", "CapacityUnits": 1.0,
            "Table": {"CapacityUnits": 1.0}}],
        "unprocessed_items": [{"Item": {"id": {"S": "a"}}}]})
    );
}

#[derive(Raiden, Debug, Clone)]
pub struct DefaultReadCompat {
    #[raiden(partition_key)]
    pub id: String,
    #[raiden(use_default)]
    pub value: String,
}

#[test]
fn derived_use_default_only_defaults_missing_or_true_null() {
    let base = || {
        let mut item = AttributeValues::new();
        item.insert("id".into(), "id".to_owned().into_attr());
        item
    };
    assert_eq!(DefaultReadCompat::from_item(base()).unwrap().value, "");
    for (attribute, expected) in [
        (null(true), Some("")),
        (null(false), None),
        ("present".to_owned().into_attr(), Some("present")),
    ] {
        let mut item = base();
        item.insert("value".into(), attribute);
        let result = DefaultReadCompat::from_item(item);
        match expected {
            Some(value) => assert_eq!(result.unwrap().value, value),
            None => assert!(result.is_err()),
        }
    }
}

#[test]
fn document_rejects_false_null_at_every_depth() {
    for attribute in [
        json!({"NULL": false}),
        json!({"L": [{"NULL": false}]}),
        json!({"M": {"nested": {"L": [{"NULL": false}]}}}),
    ] {
        let output: raiden::ops::batch_put::BatchPutOutput = serde_json::from_value(json!({
            "consumed_capacity": null, "unprocessed_items": [{"Item": {"raw": attribute}}]
        }))
        .unwrap();
        assert!(Document::<serde_json::Value>::from_attr(Some(
            output.unprocessed_items[0].item["raw"].clone()
        ))
        .is_err());
    }
    assert!(Document::<serde_json::Value>::from_attr(Some(null(true))).is_ok());
}

fn check_raw_json_boundaries(attribute: serde_json::Value, accepted: bool) {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let map = json!({"raw": attribute});
    macro_rules! check {
        ($ty:ty, $fixture:expr) => {
            assert_eq!(
                serde_json::from_value::<$ty>($fixture).is_ok(),
                accepted,
                "{}: {map}",
                stringify!($ty)
            );
        };
    }
    check!(
        raiden::ops::batch_put::BatchPutOutput,
        json!({"consumed_capacity": null, "unprocessed_items": [{"Item": map}]})
    );
    check!(
        raiden::ops::batch_delete::BatchDeleteOutput,
        json!({"consumed_capacity": null, "unprocessed_items": [{"Key": map}]})
    );
    check!(
        raiden::ops::batch_get::BatchGetOutput<serde_json::Value>,
        json!({"consumed_capacity": null, "items": [], "unprocessed_keys": {"Keys": [map]}})
    );
    check!(
        raiden::ops::update::UpdateOutput<serde_json::Value>,
        json!({"consumed_capacity": null, "item": null,
            "item_collection_metrics": {"ItemCollectionKey": map}})
    );
    check!(
        raiden::ops::scan::ScanOutput<serde_json::Value>,
        json!({"consumed_capacity": null, "items": [], "count": null,
            "scanned_count": null, "last_evaluated_key": map})
    );
    assert_eq!(
        NextToken::new(STANDARD.encode(map.to_string()))
            .into_attr_values()
            .is_ok(),
        accepted,
        "NextToken: {map}"
    );
}

#[test]
fn malformed_raw_attributes_are_rejected_at_every_json_boundary() {
    let mut invalid = vec![
        json!({}),
        json!({"UNKNOWN": "value"}),
        json!({"S": "a", "N": "1"}),
        json!({"NULL": null}),
        json!({"S": "a", "UNKNOWN": "value"}),
        json!({"B": "!invalid"}),
        json!({"S": 1}),
        json!({"NS": [1]}),
        json!({"BOOL": "false"}),
    ];
    for tag in ["B", "BOOL", "BS", "L", "M", "N", "NS", "S", "SS"] {
        invalid.push(json!({tag: null}));
    }
    for value in invalid {
        for nested in [
            value.clone(),
            json!({"L": [value]}),
            json!({"M": {"child": value}}),
            json!({"L": [{"M": {"child": value}}]}),
        ] {
            check_raw_json_boundaries(nested, false);
        }
    }
}

#[test]
fn raw_json_validation_preserves_explicit_empty_values_and_null_flags() {
    for value in [
        json!({"L": []}),
        json!({"M": {}}),
        json!({"SS": []}),
        json!({"NS": []}),
        json!({"BS": []}),
        json!({"S": ""}),
        json!({"B": ""}),
        json!({"NULL": true}),
        json!({"NULL": false}),
        json!({"BOOL": false}),
        json!({"N": "not-a-number"}),
    ] {
        check_raw_json_boundaries(value.clone(), true);
        check_raw_json_boundaries(json!({"L": [{"M": {"child": value}}]}), true);
    }
    // User model JSON is not DynamoDB AttributeValue JSON.
    let output: raiden::ops::update::UpdateOutput<serde_json::Value> =
        serde_json::from_value(json!({"consumed_capacity": null,
            "item": {"NULL": null, "S": "a", "N": "1"},
            "item_collection_metrics": null}))
        .unwrap();
    assert_eq!(
        output.item.unwrap(),
        json!({"NULL": null, "S": "a", "N": "1"})
    );
}
