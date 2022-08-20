use std::fmt;

use thiserror::Error;

const TRANSACTION_CANCELLED_MESSAGE_PREFIX: &str = "Transaction cancelled, please refer cancellation reasons for specific reasons";

#[derive(Debug, PartialEq)]
pub struct RaidenTransactionCancellationReasons(pub Vec<RaidenTransactionCancellationReason>);

impl RaidenTransactionCancellationReasons {
    // If `message` is unexcepted format, [RaidenTransactionCancellationReason::Unknown] is returned instead of Err(_)
    pub fn from_str(message: &str) -> Self {
        if !message.starts_with(TRANSACTION_CANCELLED_MESSAGE_PREFIX) {
            return RaidenTransactionCancellationReasons(vec![RaidenTransactionCancellationReason::Unknown])
        }

        RaidenTransactionCancellationReasons(message[TRANSACTION_CANCELLED_MESSAGE_PREFIX.len()..]
            .trim_matches(|c| char::is_whitespace(c) || c == '[' || c == ']')
            .split(",")
            .map(str::trim)
            .map(RaidenTransactionCancellationReason::from_str)
            .collect())
    }
}

impl fmt::Display for RaidenTransactionCancellationReasons {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reasons = self.0.iter()
            .map(|reason| reason.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        f.write_fmt(format_args!("[{}]", reasons))
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum RaidenTransactionCancellationReason {
    #[error("Unknown")]
    Unknown,
    #[error("NoError")]
    NoError,
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
            "None" => Self::NoError,
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
            RaidenTransactionCancellationReason::ConditionalCheckFailed,
        ]));
    }

    #[test]
    fn parse_message_multi() {
        let message = "Transaction cancelled, please refer cancellation reasons for specific reasons [None, ConditionalCheckFailed]";
        let reasons = RaidenTransactionCancellationReasons::from_str(message);

        assert_eq!(reasons, RaidenTransactionCancellationReasons(vec![
            RaidenTransactionCancellationReason::NoError,
            RaidenTransactionCancellationReason::ConditionalCheckFailed,
        ]));
    }

    #[test]
    fn parse_message_unknown() {
        let message = "Transaction cancelled, please refer cancellation reasons for specific reasons [UnknownSuperError]";
        let reasons = RaidenTransactionCancellationReasons::from_str(message);

        assert_eq!(reasons, RaidenTransactionCancellationReasons(vec![
            RaidenTransactionCancellationReason::Unknown,
        ]));
    }

    #[test]
    fn parse_message_unexpected_format() {
        let message = "A language empowering everyone to build reliable and efficient software";
        let reasons = RaidenTransactionCancellationReasons::from_str(message);

        assert_eq!(reasons, RaidenTransactionCancellationReasons(vec![
            RaidenTransactionCancellationReason::Unknown,
        ]));
    }
}