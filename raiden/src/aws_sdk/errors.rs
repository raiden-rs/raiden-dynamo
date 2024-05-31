use crate::{
    aws_sdk::{
        error::SdkError,
        operation::{
            batch_get_item::BatchGetItemError, batch_write_item::BatchWriteItemError,
            delete_item::DeleteItemError, get_item::GetItemError, put_item::PutItemError,
            query::QueryError, scan::ScanError, transact_write_items::TransactWriteItemsError,
            update_item::UpdateItemError,
        },
    },
    RaidenError, RaidenTransactionCancellationReasons,
};

fn into_raiden_error<E>(error: SdkError<E>) -> RaidenError {
    match error {
        SdkError::ConstructionFailure(err) => RaidenError::Construction(err),
        SdkError::TimeoutError(err) => RaidenError::Timeout(err),
        SdkError::DispatchFailure(err) => RaidenError::HttpDispatch(err),
        SdkError::ResponseError(err) => RaidenError::Unknown(err.into_raw()),
        SdkError::ServiceError(err) => {
            // SdkError::ServiceError should be handled ( except for E::Unhandled(_)).
            RaidenError::Unknown(err.into_raw())
        }
        _ => unreachable!(
            "Unexpected variant of SdkError detected. Raiden must be handle this variant."
        ),
    }
}

impl From<SdkError<BatchGetItemError>> for RaidenError {
    fn from(error: SdkError<BatchGetItemError>) -> Self {
        match &error {
            SdkError::ServiceError(err) => match err.err() {
                BatchGetItemError::InternalServerError(err) => {
                    RaidenError::InternalServerError(err.to_string())
                }
                BatchGetItemError::InvalidEndpointException(err) => {
                    RaidenError::InternalServerError(err.to_string())
                }
                BatchGetItemError::ProvisionedThroughputExceededException(err) => {
                    RaidenError::ProvisionedThroughputExceeded(err.to_string())
                }
                BatchGetItemError::RequestLimitExceeded(err) => {
                    RaidenError::RequestLimitExceeded(err.to_string())
                }
                BatchGetItemError::ResourceNotFoundException(err) => {
                    RaidenError::ResourceNotFound(err.to_string())
                }
                _ => into_raiden_error(error),
            },
            _ => into_raiden_error(error),
        }
    }
}

impl From<SdkError<BatchWriteItemError>> for RaidenError {
    fn from(error: SdkError<BatchWriteItemError>) -> Self {
        match &error {
            SdkError::ServiceError(err) => match err.err() {
                BatchWriteItemError::InternalServerError(err) => {
                    RaidenError::InternalServerError(err.to_string())
                }
                BatchWriteItemError::InvalidEndpointException(err) => {
                    RaidenError::InternalServerError(err.to_string())
                }
                BatchWriteItemError::ItemCollectionSizeLimitExceededException(err) => {
                    RaidenError::ItemCollectionSizeLimitExceeded(err.to_string())
                }
                BatchWriteItemError::ProvisionedThroughputExceededException(err) => {
                    RaidenError::ProvisionedThroughputExceeded(err.to_string())
                }
                BatchWriteItemError::RequestLimitExceeded(err) => {
                    RaidenError::RequestLimitExceeded(err.to_string())
                }
                BatchWriteItemError::ResourceNotFoundException(err) => {
                    RaidenError::ResourceNotFound(err.to_string())
                }
                _ => into_raiden_error(error),
            },
            _ => into_raiden_error(error),
        }
    }
}

impl From<SdkError<GetItemError>> for RaidenError {
    fn from(error: SdkError<GetItemError>) -> Self {
        match &error {
            SdkError::ServiceError(err) => match err.err() {
                GetItemError::InternalServerError(err) => {
                    RaidenError::InternalServerError(err.to_string())
                }
                GetItemError::InvalidEndpointException(err) => {
                    RaidenError::InternalServerError(err.to_string())
                }
                GetItemError::ProvisionedThroughputExceededException(err) => {
                    RaidenError::ProvisionedThroughputExceeded(err.to_string())
                }
                GetItemError::RequestLimitExceeded(err) => {
                    RaidenError::RequestLimitExceeded(err.to_string())
                }
                GetItemError::ResourceNotFoundException(err) => {
                    RaidenError::ResourceNotFound(err.to_string())
                }
                _ => into_raiden_error(error),
            },
            _ => into_raiden_error(error),
        }
    }
}

impl From<SdkError<QueryError>> for RaidenError {
    fn from(error: SdkError<QueryError>) -> Self {
        match &error {
            SdkError::ServiceError(err) => match err.err() {
                QueryError::InternalServerError(err) => {
                    RaidenError::InternalServerError(err.to_string())
                }
                QueryError::InvalidEndpointException(err) => {
                    RaidenError::InternalServerError(err.to_string())
                }
                QueryError::ProvisionedThroughputExceededException(err) => {
                    RaidenError::ProvisionedThroughputExceeded(err.to_string())
                }
                QueryError::RequestLimitExceeded(err) => {
                    RaidenError::RequestLimitExceeded(err.to_string())
                }
                QueryError::ResourceNotFoundException(err) => {
                    RaidenError::ResourceNotFound(err.to_string())
                }
                _ => into_raiden_error(error),
            },
            _ => into_raiden_error(error),
        }
    }
}

impl From<SdkError<ScanError>> for RaidenError {
    fn from(error: SdkError<ScanError>) -> Self {
        match &error {
            SdkError::ServiceError(err) => match err.err() {
                ScanError::InternalServerError(err) => {
                    RaidenError::InternalServerError(err.to_string())
                }
                ScanError::InvalidEndpointException(err) => {
                    RaidenError::InternalServerError(err.to_string())
                }
                ScanError::ProvisionedThroughputExceededException(err) => {
                    RaidenError::ProvisionedThroughputExceeded(err.to_string())
                }
                ScanError::RequestLimitExceeded(err) => {
                    RaidenError::RequestLimitExceeded(err.to_string())
                }
                ScanError::ResourceNotFoundException(err) => {
                    RaidenError::ResourceNotFound(err.to_string())
                }
                _ => into_raiden_error(error),
            },
            _ => into_raiden_error(error),
        }
    }
}

impl From<SdkError<PutItemError>> for RaidenError {
    fn from(error: SdkError<PutItemError>) -> Self {
        match &error {
            SdkError::ServiceError(err) => match err.err() {
                PutItemError::ConditionalCheckFailedException(err) => {
                    RaidenError::ConditionalCheckFailed(err.to_string())
                }
                PutItemError::InternalServerError(err) => {
                    RaidenError::InternalServerError(err.to_string())
                }
                PutItemError::InvalidEndpointException(err) => {
                    RaidenError::InternalServerError(err.to_string())
                }
                PutItemError::ItemCollectionSizeLimitExceededException(err) => {
                    RaidenError::ItemCollectionSizeLimitExceeded(err.to_string())
                }
                PutItemError::ProvisionedThroughputExceededException(err) => {
                    RaidenError::ProvisionedThroughputExceeded(err.to_string())
                }
                PutItemError::RequestLimitExceeded(err) => {
                    RaidenError::RequestLimitExceeded(err.to_string())
                }
                PutItemError::ResourceNotFoundException(err) => {
                    RaidenError::ResourceNotFound(err.to_string())
                }
                PutItemError::TransactionConflictException(err) => {
                    RaidenError::TransactionConflict(err.to_string())
                }
                _ => into_raiden_error(error),
            },
            _ => into_raiden_error(error),
        }
    }
}

impl From<SdkError<UpdateItemError>> for RaidenError {
    fn from(error: SdkError<UpdateItemError>) -> Self {
        match &error {
            SdkError::ServiceError(err) => match err.err() {
                UpdateItemError::ConditionalCheckFailedException(err) => {
                    RaidenError::ConditionalCheckFailed(err.to_string())
                }
                UpdateItemError::InternalServerError(err) => {
                    RaidenError::InternalServerError(err.to_string())
                }
                UpdateItemError::InvalidEndpointException(err) => {
                    RaidenError::InternalServerError(err.to_string())
                }
                UpdateItemError::ItemCollectionSizeLimitExceededException(err) => {
                    RaidenError::ItemCollectionSizeLimitExceeded(err.to_string())
                }
                UpdateItemError::ProvisionedThroughputExceededException(err) => {
                    RaidenError::ProvisionedThroughputExceeded(err.to_string())
                }
                UpdateItemError::RequestLimitExceeded(err) => {
                    RaidenError::RequestLimitExceeded(err.to_string())
                }
                UpdateItemError::ResourceNotFoundException(err) => {
                    RaidenError::ResourceNotFound(err.to_string())
                }
                UpdateItemError::TransactionConflictException(err) => {
                    RaidenError::TransactionConflict(err.to_string())
                }
                _ => into_raiden_error(error),
            },
            _ => into_raiden_error(error),
        }
    }
}

impl From<SdkError<DeleteItemError>> for RaidenError {
    fn from(error: SdkError<DeleteItemError>) -> Self {
        match &error {
            SdkError::ServiceError(err) => match err.err() {
                DeleteItemError::ConditionalCheckFailedException(err) => {
                    RaidenError::ConditionalCheckFailed(err.to_string())
                }
                DeleteItemError::InternalServerError(err) => {
                    RaidenError::InternalServerError(err.to_string())
                }
                DeleteItemError::InvalidEndpointException(err) => {
                    RaidenError::InternalServerError(err.to_string())
                }
                DeleteItemError::ItemCollectionSizeLimitExceededException(err) => {
                    RaidenError::ItemCollectionSizeLimitExceeded(err.to_string())
                }
                DeleteItemError::ProvisionedThroughputExceededException(err) => {
                    RaidenError::ProvisionedThroughputExceeded(err.to_string())
                }
                DeleteItemError::RequestLimitExceeded(err) => {
                    RaidenError::RequestLimitExceeded(err.to_string())
                }
                DeleteItemError::ResourceNotFoundException(err) => {
                    RaidenError::ResourceNotFound(err.to_string())
                }
                DeleteItemError::TransactionConflictException(err) => {
                    RaidenError::TransactionConflict(err.to_string())
                }
                _ => into_raiden_error(error),
            },
            _ => into_raiden_error(error),
        }
    }
}

impl From<SdkError<TransactWriteItemsError>> for RaidenError {
    fn from(error: SdkError<TransactWriteItemsError>) -> Self {
        match &error {
            SdkError::ServiceError(err) => match err.err() {
                TransactWriteItemsError::IdempotentParameterMismatchException(err) => {
                    RaidenError::IdempotentParameterMismatch(err.to_string())
                }
                TransactWriteItemsError::InternalServerError(err) => {
                    RaidenError::InternalServerError(err.to_string())
                }
                TransactWriteItemsError::InvalidEndpointException(err) => {
                    RaidenError::InternalServerError(err.to_string())
                }
                TransactWriteItemsError::ProvisionedThroughputExceededException(err) => {
                    RaidenError::ProvisionedThroughputExceeded(err.to_string())
                }
                TransactWriteItemsError::RequestLimitExceeded(err) => {
                    RaidenError::RequestLimitExceeded(err.to_string())
                }
                TransactWriteItemsError::ResourceNotFoundException(err) => {
                    RaidenError::ResourceNotFound(err.to_string())
                }
                TransactWriteItemsError::TransactionCanceledException(err) => {
                    let reasons = RaidenTransactionCancellationReasons::from_str(
                        err.message
                            .clone()
                            .unwrap_or_else(|| "transaction canceled".to_owned())
                            .as_str(),
                    );
                    let raw_reasons = err.cancellation_reasons.clone().unwrap_or_default();

                    RaidenError::TransactionCanceled {
                        reasons,
                        raw_reasons,
                    }
                }
                TransactWriteItemsError::TransactionInProgressException(err) => {
                    RaidenError::TransactionInProgress(err.to_string())
                }
                _ => into_raiden_error(error),
            },
            _ => into_raiden_error(error),
        }
    }
}
