//! Shared fixtures make the two backends obey the same attribute contract.
use raiden::*;
use serde_json::json;
use std::borrow::Cow;
use std::collections::{BTreeSet, HashSet};

fn null(value: bool) -> AttributeValue {
    #[cfg(feature = "aws-sdk")]
    return AttributeValue::Null(value);
    #[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
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
    let fixture = json!({"consumed_capacity": null, "unprocessed_items": [{"item": attributes}]});
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
            "unprocessed_items": [{"item": {"null": {"NULL": flag}}}]});
        let output: raiden::ops::batch_put::BatchPutOutput =
            serde_json::from_value(fixture.clone()).unwrap();
        assert_eq!(serde_json::to_value(output).unwrap(), fixture);
    }
}

#[test]
fn batch_put_capacity_metadata_round_trips_with_optional_null_fields() {
    let fixture = json!({"consumed_capacity": [{
        "table_name": "items", "capacity_units": 1.0,
        "read_capacity_units": null, "write_capacity_units": null,
        "table": {"capacity_units": 1.0, "read_capacity_units": null, "write_capacity_units": null},
        "local_secondary_indexes": null, "global_secondary_indexes": null
    }], "unprocessed_items": []});
    let output: raiden::ops::batch_put::BatchPutOutput =
        serde_json::from_value(fixture.clone()).unwrap();
    assert_eq!(serde_json::to_value(output).unwrap(), fixture);
    let fixture = json!({"consumed_capacity": [{
        "table_name": null, "capacity_units": null,
        "read_capacity_units": null, "write_capacity_units": null,
        "table": null, "local_secondary_indexes": null, "global_secondary_indexes": null
    }], "unprocessed_items": []});
    let output: raiden::ops::batch_put::BatchPutOutput =
        serde_json::from_value(fixture.clone()).unwrap();
    assert_eq!(serde_json::to_value(output).unwrap(), fixture);
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
