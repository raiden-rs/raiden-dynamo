#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
use crate::{ConsumedCapacity, KeysAndAttributes};

#[cfg(feature = "aws-sdk")]
use crate::aws_sdk::types::{ConsumedCapacity, KeysAndAttributes};

// See. https://github.com/rusoto/rusoto/blob/69e7c9150d98916ef8fc814f5cd17eb0e4dee3d3/rusoto/services/dynamodb/src/generated.rs#L356
#[derive(Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    any(feature = "rusoto", feature = "rusoto_rustls"),
    derive(serde::Deserialize, serde::Serialize)
)]
pub struct BatchGetOutput<T> {
    pub consumed_capacity: Option<Vec<ConsumedCapacity>>,
    pub items: Vec<T>,
    pub unprocessed_keys: Option<KeysAndAttributes>,
}

#[cfg(test)]
mod tests {
    use super::dedupe_keys;
    use crate::AttributeValue;

    #[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
    mod attr {
        use crate::AttributeValue;

        pub fn s(v: &str) -> AttributeValue {
            AttributeValue {
                s: Some(v.to_owned()),
                ..AttributeValue::default()
            }
        }

        pub fn n(v: &str) -> AttributeValue {
            AttributeValue {
                n: Some(v.to_owned()),
                ..AttributeValue::default()
            }
        }

        pub fn b(v: &[u8]) -> AttributeValue {
            AttributeValue {
                b: Some(v.to_vec().into()),
                ..AttributeValue::default()
            }
        }

        pub fn bool(v: bool) -> AttributeValue {
            AttributeValue {
                bool: Some(v),
                ..AttributeValue::default()
            }
        }
    }

    #[cfg(feature = "aws-sdk")]
    mod attr {
        use crate::AttributeValue;

        pub fn s(v: &str) -> AttributeValue {
            AttributeValue::S(v.to_owned())
        }

        pub fn n(v: &str) -> AttributeValue {
            AttributeValue::N(v.to_owned())
        }

        pub fn b(v: &[u8]) -> AttributeValue {
            AttributeValue::B(crate::aws_sdk::primitives::Blob::new(v.to_vec()))
        }

        pub fn bool(v: bool) -> AttributeValue {
            AttributeValue::Bool(v)
        }
    }

    use attr::*;

    #[test]
    fn keeps_first_occurrence_order() {
        let keys = vec![s("c"), s("a"), s("c"), s("b"), s("a")];
        assert_eq!(dedupe_keys(keys), vec![s("c"), s("a"), s("b")]);
    }

    #[test]
    fn empty_input() {
        assert_eq!(dedupe_keys(Vec::<AttributeValue>::new()), vec![]);
    }

    #[test]
    fn all_duplicates() {
        let keys = vec![s("a"), s("a"), s("a")];
        assert_eq!(dedupe_keys(keys), vec![s("a")]);
    }

    #[test]
    fn scalar_types_are_distinct() {
        // The same text as S and N is a different key type, and B compares by bytes.
        let keys = vec![s("1"), n("1"), b(b"1"), n("1"), b(b"1"), s("1"), b(b"2")];
        assert_eq!(dedupe_keys(keys), vec![s("1"), n("1"), b(b"1"), b(b"2")]);
    }

    #[test]
    fn non_key_types_pass_through() {
        // Only S, N and B can be key attributes. Anything else is sent as is so
        // that DynamoDB reports the invalid key instead of raiden hiding it.
        let keys = vec![bool(true), bool(true), s("a"), s("a")];
        assert_eq!(dedupe_keys(keys), vec![bool(true), bool(true), s("a")]);
    }

    #[test]
    fn composite_keys() {
        let keys = vec![
            (s("a"), n("1")),
            (s("a"), n("2")),
            (s("a"), n("1")),
            (s("b"), n("1")),
            (s("b"), n("1")),
        ];
        assert_eq!(
            dedupe_keys(keys),
            vec![(s("a"), n("1")), (s("a"), n("2")), (s("b"), n("1"))]
        );
    }

    #[test]
    fn composite_keys_with_non_key_type_pass_through() {
        let keys = vec![(s("a"), bool(true)), (s("a"), bool(true))];
        assert_eq!(
            dedupe_keys(keys),
            vec![(s("a"), bool(true)), (s("a"), bool(true))]
        );
    }
}
