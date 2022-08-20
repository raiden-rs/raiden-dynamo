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

    // as_error returns first error or Unknow
    pub fn to_error(&self) -> RaidenTransactionCancellationReason {
        for reason in &self.0 {
            match reason {
                Some(reason) => return reason.clone(),
                None => {}
            }
        }
        RaidenTransactionCancellationReason::Unknown
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
    fn to_error() {
        let results = RaidenTransactionCancellationReasons(vec![
            None,
            Some(RaidenTransactionCancellationReason::ConditionalCheckFailed),
        ]);
        assert_eq!(results.to_error(), RaidenTransactionCancellationReason::ConditionalCheckFailed);
    }

    #[test]
    fn to_error_none() {
        // Usually, AWS doesn't return such results, so to_error should return RaidenTransactionCancellationReason::Unknown
        let results = RaidenTransactionCancellationReasons(vec![
            None,
        ]);
        assert_eq!(results.to_error(), RaidenTransactionCancellationReason::Unknown);
    }

    #[test]
    fn to_error_empty() {
        // Usually, AWS doesn't return such results, so to_error should return RaidenTransactionCancellationReason::Unknown
        let results = RaidenTransactionCancellationReasons(vec![
        ]);
        assert_eq!(results.to_error(), RaidenTransactionCancellationReason::Unknown);
    }
}