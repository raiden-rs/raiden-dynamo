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

/// A hashable form of a DynamoDB key attribute. Key attributes can only be
/// strings, numbers or binaries.
#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KeyRepr {
    S(String),
    N(String),
    B(Vec<u8>),
}

/// Keys accepted by the generated `batch_get`: a partition key, or a pair of
/// partition key and sort key.
#[doc(hidden)]
pub trait BatchGetKey {
    type Repr: Eq + std::hash::Hash;

    /// Returns `None` when the key contains a value that cannot be a DynamoDB
    /// key attribute. Such keys are never removed as duplicates.
    fn key_repr(&self) -> Option<Self::Repr>;
}

#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
impl BatchGetKey for crate::AttributeValue {
    type Repr = KeyRepr;

    fn key_repr(&self) -> Option<KeyRepr> {
        let crate::AttributeValue {
            b,
            bool,
            bs,
            l,
            m,
            n,
            ns,
            null,
            s,
            ss,
        } = self;
        if bool.is_some()
            || bs.is_some()
            || l.is_some()
            || m.is_some()
            || ns.is_some()
            || null.is_some()
            || ss.is_some()
        {
            return None;
        }
        match (s, n, b) {
            (Some(s), None, None) => Some(KeyRepr::S(s.clone())),
            (None, Some(n), None) => Some(KeyRepr::N(n.clone())),
            (None, None, Some(b)) => Some(KeyRepr::B(b.to_vec())),
            _ => None,
        }
    }
}

#[cfg(feature = "aws-sdk")]
impl BatchGetKey for crate::AttributeValue {
    type Repr = KeyRepr;

    fn key_repr(&self) -> Option<KeyRepr> {
        match self {
            Self::S(s) => Some(KeyRepr::S(s.clone())),
            Self::N(n) => Some(KeyRepr::N(n.clone())),
            Self::B(b) => Some(KeyRepr::B(b.as_ref().to_vec())),
            _ => None,
        }
    }
}

impl<P: BatchGetKey, S: BatchGetKey> BatchGetKey for (P, S) {
    type Repr = (P::Repr, S::Repr);

    fn key_repr(&self) -> Option<Self::Repr> {
        Some((self.0.key_repr()?, self.1.key_repr()?))
    }
}

/// Removes repeated keys, keeping the first occurrence of each key in order.
///
/// BatchGetItem rejects a request that contains the same key twice, and the
/// same key split into different requests would return the item twice.
#[doc(hidden)]
pub fn dedupe_keys<K: BatchGetKey>(keys: Vec<K>) -> Vec<K> {
    let mut seen = std::collections::HashSet::with_capacity(keys.len());
    keys.into_iter()
        .filter(|key| match key.key_repr() {
            Some(repr) => seen.insert(repr),
            None => true,
        })
        .collect()
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
