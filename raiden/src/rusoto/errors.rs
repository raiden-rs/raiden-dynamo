use crate::*;

fn into_raiden_error<E>(err: RusotoError<E>) -> RaidenError {
    match err {
        RusotoError::HttpDispatch(err) => RaidenError::HttpDispatch(err),
        RusotoError::Credentials(err) => RaidenError::Credentials(err),
        RusotoError::Validation(msg) => RaidenError::Validation(msg),
        RusotoError::ParseError(msg) => RaidenError::ParseError(msg),
        RusotoError::Unknown(res) => RaidenError::Unknown(res),
        RusotoError::Blocking => RaidenError::Blocking,
        RusotoError::Service(_) => unimplemented!("RusotoError::Service should be handled."),
    }
}

impl From<RusotoError<BatchGetItemError>> for RaidenError {
    fn from(error: RusotoError<BatchGetItemError>) -> Self {
        match error {
            RusotoError::Service(error) => match error {
                BatchGetItemError::InternalServerError(msg) => {
                    RaidenError::InternalServerError(msg)
                }
                BatchGetItemError::ProvisionedThroughputExceeded(msg) => {
                    RaidenError::ProvisionedThroughputExceeded(msg)
                }
                BatchGetItemError::RequestLimitExceeded(msg) => {
                    RaidenError::RequestLimitExceeded(msg)
                }
                BatchGetItemError::ResourceNotFound(msg) => RaidenError::ResourceNotFound(msg),
            },
            _ => into_raiden_error(error),
        }
    }
}

impl From<RusotoError<BatchWriteItemError>> for RaidenError {
    fn from(error: RusotoError<BatchWriteItemError>) -> Self {
        match error {
            RusotoError::Service(error) => match error {
                BatchWriteItemError::InternalServerError(msg) => {
                    RaidenError::InternalServerError(msg)
                }
                BatchWriteItemError::ItemCollectionSizeLimitExceeded(msg) => {
                    RaidenError::ItemCollectionSizeLimitExceeded(msg)
                }
                BatchWriteItemError::ProvisionedThroughputExceeded(msg) => {
                    RaidenError::ProvisionedThroughputExceeded(msg)
                }
                BatchWriteItemError::RequestLimitExceeded(msg) => {
                    RaidenError::RequestLimitExceeded(msg)
                }
                BatchWriteItemError::ResourceNotFound(msg) => RaidenError::ResourceNotFound(msg),
            },
            _ => into_raiden_error(error),
        }
    }
}

impl From<RusotoError<GetItemError>> for RaidenError {
    fn from(error: RusotoError<GetItemError>) -> Self {
        match error {
            RusotoError::Service(error) => match error {
                GetItemError::InternalServerError(msg) => RaidenError::InternalServerError(msg),
                GetItemError::ProvisionedThroughputExceeded(msg) => {
                    RaidenError::ProvisionedThroughputExceeded(msg)
                }
                GetItemError::RequestLimitExceeded(msg) => RaidenError::RequestLimitExceeded(msg),
                GetItemError::ResourceNotFound(msg) => RaidenError::ResourceNotFound(msg),
            },
            _ => into_raiden_error(error),
        }
    }
}

impl From<RusotoError<QueryError>> for RaidenError {
    fn from(error: RusotoError<QueryError>) -> Self {
        match error {
            RusotoError::Service(error) => match error {
                QueryError::InternalServerError(msg) => RaidenError::InternalServerError(msg),
                QueryError::ProvisionedThroughputExceeded(msg) => {
                    RaidenError::ProvisionedThroughputExceeded(msg)
                }
                QueryError::RequestLimitExceeded(msg) => RaidenError::RequestLimitExceeded(msg),
                QueryError::ResourceNotFound(msg) => RaidenError::ResourceNotFound(msg),
            },
            _ => into_raiden_error(error),
        }
    }
}

impl From<RusotoError<ScanError>> for RaidenError {
    fn from(error: RusotoError<ScanError>) -> Self {
        match error {
            RusotoError::Service(error) => match error {
                ScanError::InternalServerError(msg) => RaidenError::InternalServerError(msg),
                ScanError::ProvisionedThroughputExceeded(msg) => {
                    RaidenError::ProvisionedThroughputExceeded(msg)
                }
                ScanError::RequestLimitExceeded(msg) => RaidenError::RequestLimitExceeded(msg),
                ScanError::ResourceNotFound(msg) => RaidenError::ResourceNotFound(msg),
            },
            _ => into_raiden_error(error),
        }
    }
}

impl From<RusotoError<PutItemError>> for RaidenError {
    fn from(error: RusotoError<PutItemError>) -> Self {
        match error {
            RusotoError::Service(error) => match error {
                PutItemError::InternalServerError(msg) => RaidenError::InternalServerError(msg),
                PutItemError::ProvisionedThroughputExceeded(msg) => {
                    RaidenError::ProvisionedThroughputExceeded(msg)
                }
                PutItemError::RequestLimitExceeded(msg) => RaidenError::RequestLimitExceeded(msg),
                PutItemError::ResourceNotFound(msg) => RaidenError::ResourceNotFound(msg),
                PutItemError::ConditionalCheckFailed(msg) => {
                    RaidenError::ConditionalCheckFailed(msg)
                }
                PutItemError::ItemCollectionSizeLimitExceeded(msg) => {
                    RaidenError::ItemCollectionSizeLimitExceeded(msg)
                }
                PutItemError::TransactionConflict(msg) => RaidenError::TransactionConflict(msg),
            },
            _ => into_raiden_error(error),
        }
    }
}

impl From<RusotoError<UpdateItemError>> for RaidenError {
    fn from(error: RusotoError<UpdateItemError>) -> Self {
        match error {
            RusotoError::Service(error) => match error {
                UpdateItemError::InternalServerError(msg) => RaidenError::InternalServerError(msg),
                UpdateItemError::ProvisionedThroughputExceeded(msg) => {
                    RaidenError::ProvisionedThroughputExceeded(msg)
                }
                UpdateItemError::RequestLimitExceeded(msg) => {
                    RaidenError::RequestLimitExceeded(msg)
                }
                UpdateItemError::ResourceNotFound(msg) => RaidenError::ResourceNotFound(msg),
                UpdateItemError::ConditionalCheckFailed(msg) => {
                    RaidenError::ConditionalCheckFailed(msg)
                }
                UpdateItemError::ItemCollectionSizeLimitExceeded(msg) => {
                    RaidenError::ItemCollectionSizeLimitExceeded(msg)
                }
                UpdateItemError::TransactionConflict(msg) => RaidenError::TransactionConflict(msg),
            },
            _ => into_raiden_error(error),
        }
    }
}

impl From<RusotoError<DeleteItemError>> for RaidenError {
    fn from(error: RusotoError<DeleteItemError>) -> Self {
        match error {
            RusotoError::Service(error) => match error {
                DeleteItemError::InternalServerError(msg) => RaidenError::InternalServerError(msg),
                DeleteItemError::ProvisionedThroughputExceeded(msg) => {
                    RaidenError::ProvisionedThroughputExceeded(msg)
                }
                DeleteItemError::RequestLimitExceeded(msg) => {
                    RaidenError::RequestLimitExceeded(msg)
                }
                DeleteItemError::ResourceNotFound(msg) => RaidenError::ResourceNotFound(msg),
                DeleteItemError::ConditionalCheckFailed(msg) => {
                    RaidenError::ConditionalCheckFailed(msg)
                }
                DeleteItemError::ItemCollectionSizeLimitExceeded(msg) => {
                    RaidenError::ItemCollectionSizeLimitExceeded(msg)
                }
                DeleteItemError::TransactionConflict(msg) => RaidenError::TransactionConflict(msg),
            },
            _ => into_raiden_error(error),
        }
    }
}

impl From<RusotoError<TransactWriteItemsError>> for RaidenError {
    fn from(error: RusotoError<TransactWriteItemsError>) -> Self {
        match error {
            RusotoError::Service(error) => match error {
                TransactWriteItemsError::IdempotentParameterMismatch(msg) => {
                    RaidenError::IdempotentParameterMismatch(msg)
                }
                TransactWriteItemsError::InternalServerError(msg) => {
                    RaidenError::InternalServerError(msg)
                }
                TransactWriteItemsError::ProvisionedThroughputExceeded(msg) => {
                    RaidenError::ProvisionedThroughputExceeded(msg)
                }
                TransactWriteItemsError::RequestLimitExceeded(msg) => {
                    RaidenError::RequestLimitExceeded(msg)
                }
                TransactWriteItemsError::ResourceNotFound(msg) => {
                    RaidenError::ResourceNotFound(msg)
                }
                TransactWriteItemsError::TransactionCanceled(msg) => {
                    let reasons = RaidenTransactionCancellationReasons::from_str(&msg);
                    RaidenError::TransactionCanceled { reasons }
                }
                TransactWriteItemsError::TransactionInProgress(msg) => {
                    RaidenError::TransactionInProgress(msg)
                }
            },
            _ => into_raiden_error(error),
        }
    }
}
