use std::fmt;

use thiserror::Error;

const TRANSACTION_CANCELLED_MESSAGE_PREFIX: &str = "Transaction cancelled, please refer cancellation reasons for specific reasons";

#[derive(Clone, Debug, PartialEq)]
pub struct RaidenTransactionCancellationReasons(pub Vec<Option<RaidenTransactionCancellationReason>>);

impl RaidenTransactionCancellationReasons {
    // If `message` is unexcepted format, [RaidenTransactionCancellationReason::Unknown] is returned instead of Err(_)
    pub fn from_str(message: &str) -> Self {
        if !message.starts_with(TRANSACTION_CANCELLED_MESSAGE_PREFIX) {
            return RaidenTransactionCancellationReasons(vec![
                Some(RaidenTransactionCancellationReason::Unknown)
            ])
        }

        RaidenTransactionCancellationReasons(message[TRANSACTION_CANCELLED_MESSAGE_PREFIX.len()..]
            .trim_matches(|c| char::is_whitespace(c) || c == '[' || c == ']')
            .split(",")
            .map(str::trim)
            .map(|reason| {
                match reason {
                    "None" => None,
                    reason => Some(RaidenTransactionCancellationReason::from_str(reason)),
                }
            })
            .collect())
    }

    fn has_error(&self, r: RaidenTransactionCancellationReason) -> bool {
        return self.0.iter()
            .any(|reason| match reason {
                Some(error) if *error == r => return true,
                _ => return false,
            })
    }

    pub fn has_conditional_check_failed(&self) -> bool {
        return self.has_error(RaidenTransactionCancellationReason::ConditionalCheckFailed)
    }

    pub fn has_item_collection_size_limit_exceeded(&self) -> bool {
        return self.has_error(RaidenTransactionCancellationReason::ItemCollectionSizeLimitExceeded)
    }

    pub fn has_transaction_conflict(&self) -> bool {
        return self.has_error(RaidenTransactionCancellationReason::TransactionConflict)
    }

    pub fn has_provisioned_throughput_exceeded(&self) -> bool {
        return self.has_error(RaidenTransactionCancellationReason::ProvisionedThroughputExceeded)
    }

    pub fn has_throttling_error(&self) -> bool {
        return self.has_error(RaidenTransactionCancellationReason::ThrottlingError)
    }

    pub fn has_validation_error(&self) -> bool {
        return self.has_error(RaidenTransactionCancellationReason::ValidationError)
    }
}

impl fmt::Display for RaidenTransactionCancellationReasons {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reasons = self.0.iter()
            .map(|reason| {
                match reason {
                    Some(reason) => reason.to_string(),
                    None => String::from("None"),
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        f.write_fmt(format_args!("[{}]", reasons))
    }
}

#[derive(Error, Clone, Debug, PartialEq)]
pub enum RaidenTransactionCancellationReason {
    #[error("Unknown")]
    Unknown,
    #[error("ConditionalCheckFailed")]
    ConditionalCheckFailed,
    #[error("ItemCollectionSizeLimitExceeded")]
    ItemCollectionSizeLimitExceeded,
    #[error("TransactionConflict")]
    TransactionConflict,
    #[error("ProvisionedThroughputExceeded")]
    ProvisionedThroughputExceeded,
    #[error("ThrottlingError")]
    ThrottlingError,
    #[error("ValidationError")]
    ValidationError
}

impl RaidenTransactionCancellationReason {
    pub fn from_str(reason: &str) -> Self {
        match reason {
            "ConditionalCheckFailed" => Self::ConditionalCheckFailed,
            "ItemCollectionSizeLimitExceeded" => Self::ItemCollectionSizeLimitExceeded,
            "TransactionConflict" => Self::TransactionConflict,
            "ProvisionedThroughputExceeded" => Self::ProvisionedThroughputExceeded,
            "ThrottlingError" => Self::ThrottlingError,
            "ValidationError" => Self::ValidationError,
            // If `reason` is unexcepted, Self::Unknown is returned instead of Err(_)
            _ => Self::Unknown
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{RaidenTransactionCancellationReasons, RaidenTransactionCancellationReason};

    #[test]
    fn parse_message_single() {
        let message = "Transaction cancelled, please refer cancellation reasons for specific reasons [ConditionalCheckFailed]";
        let reasons = RaidenTransactionCancellationReasons::from_str(message);

        assert_eq!(reasons, RaidenTransactionCancellationReasons(vec![
            Some(RaidenTransactionCancellationReason::ConditionalCheckFailed),
        ]));
    }

    #[test]
    fn parse_message_multi() {
        let message = "Transaction cancelled, please refer cancellation reasons for specific reasons [None, ConditionalCheckFailed]";
        let reasons = RaidenTransactionCancellationReasons::from_str(message);

        assert_eq!(reasons, RaidenTransactionCancellationReasons(vec![
            None,
            Some(RaidenTransactionCancellationReason::ConditionalCheckFailed),
        ]));
    }

    #[test]
    fn parse_message_unknown() {
        let message = "Transaction cancelled, please refer cancellation reasons for specific reasons [UnknownSuperError]";
        let reasons = RaidenTransactionCancellationReasons::from_str(message);

        assert_eq!(reasons, RaidenTransactionCancellationReasons(vec![
            Some(RaidenTransactionCancellationReason::Unknown),
        ]));
    }

    #[test]
    fn parse_message_unexpected_format() {
        let message = "A language empowering everyone to build reliable and efficient software";
        let reasons = RaidenTransactionCancellationReasons::from_str(message);

        assert_eq!(reasons, RaidenTransactionCancellationReasons(vec![
            Some(RaidenTransactionCancellationReason::Unknown),
        ]));
    }

    #[test]
    fn has_error() {
        let results = RaidenTransactionCancellationReasons(vec![
            None,
            Some(RaidenTransactionCancellationReason::TransactionConflict),
            Some(RaidenTransactionCancellationReason::ConditionalCheckFailed),
        ]);

        assert!(results.has_error(RaidenTransactionCancellationReason::ConditionalCheckFailed));
        assert!(results.has_error(RaidenTransactionCancellationReason::TransactionConflict));

        assert!(!results.has_error(RaidenTransactionCancellationReason::ItemCollectionSizeLimitExceeded));
        assert!(!results.has_error(RaidenTransactionCancellationReason::ProvisionedThroughputExceeded));
        assert!(!results.has_error(RaidenTransactionCancellationReason::ThrottlingError));
        assert!(!results.has_error(RaidenTransactionCancellationReason::ValidationError));
    }
}
