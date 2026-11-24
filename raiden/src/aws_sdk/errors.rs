use crate::{
    aws_sdk::{
        error::SdkError,
        operation::{
            batch_get_item::BatchGetItemError, batch_write_item::BatchWriteItemError,
            delete_item::DeleteItemError, get_item::GetItemError, put_item::PutItemError,
            query::QueryError, scan::ScanError, transact_get_items::TransactGetItemsError,
            transact_write_items::TransactWriteItemsError, update_item::UpdateItemError,
        },
    },
    RaidenError, RaidenTransactionCancellationReasons,
};

// Match rusoto_core::proto::json::Error::parse, including its strict optional
// string fields, Message alias, last-# type suffix, and empty default message.
// Decode the HTTP body rather than SDK metadata: the SDK also accepts code/Code,
// prefers them to __type, and tolerates some malformed messages. Those choices
// otherwise change Raiden variants and application retry decisions on migration.
#[derive(serde::Deserialize)]
struct LegacyErrorBody {
    #[serde(rename = "__type", default)]
    typ: Option<String>,
    #[serde(alias = "Message", default)]
    message: Option<String>,
}

fn service_response(
    response: aws_smithy_runtime_api::client::orchestrator::HttpResponse,
    supported: &[&str],
    raw_reasons: Vec<crate::aws_sdk::types::CancellationReason>,
) -> RaidenError {
    let parsed = response
        .body()
        .bytes()
        .and_then(|body| serde_json::from_slice::<LegacyErrorBody>(body).ok());
    let Some(parsed) = parsed else {
        return RaidenError::Unknown(response);
    };
    let code = parsed
        .typ
        .as_deref()
        .unwrap_or("Unknown")
        .rsplit('#')
        .next()
        .unwrap_or("Unknown");
    // ValidationException is handled by every Rusoto DynamoDB operation.
    if code != "ValidationException" && !supported.contains(&code) {
        return RaidenError::Unknown(response);
    }
    let message = parsed.message.unwrap_or_default();
    match code {
        "ConditionalCheckFailedException" => RaidenError::ConditionalCheckFailed(message),
        "IdempotentParameterMismatchException" => RaidenError::IdempotentParameterMismatch(message),
        "InternalServerError" => RaidenError::InternalServerError(message),
        "ItemCollectionSizeLimitExceededException" => {
            RaidenError::ItemCollectionSizeLimitExceeded(message)
        }
        "ProvisionedThroughputExceededException" => {
            RaidenError::ProvisionedThroughputExceeded(message)
        }
        "RequestLimitExceeded" => RaidenError::RequestLimitExceeded(message),
        "ResourceNotFoundException" => RaidenError::ResourceNotFound(message),
        "TransactionConflictException" => RaidenError::TransactionConflict(message),
        "TransactionInProgressException" => RaidenError::TransactionInProgress(message),
        "ValidationException" => RaidenError::Validation(message),
        "TransactionCanceledException" => RaidenError::TransactionCanceled {
            reasons: RaidenTransactionCancellationReasons::from_str(&message),
            raw_reasons,
        },
        _ => RaidenError::Unknown(response),
    }
}

fn into_raiden_error<E>(
    error: SdkError<E>,
    supported: &[&str],
    cancellation_reasons: impl FnOnce(&E) -> Vec<crate::aws_sdk::types::CancellationReason>,
) -> RaidenError {
    match error {
        SdkError::ConstructionFailure(err) => RaidenError::Construction(err),
        SdkError::TimeoutError(err) => RaidenError::Timeout(err),
        SdkError::DispatchFailure(err) => RaidenError::HttpDispatch(err),
        // Invalid structured error details may make the SDK decoder fail even
        // though Rusoto can classify __type/message. Preserve that classification.
        SdkError::ResponseError(err) => service_response(err.into_raw(), supported, vec![]),
        SdkError::ServiceError(err) => {
            let reasons = cancellation_reasons(err.err());
            service_response(err.into_raw(), supported, reasons)
        }
        _ => unreachable!("Unexpected variant of SdkError detected"),
    }
}

macro_rules! impl_service_error {
    ($error:ty, [$($code:literal),* $(,)?], $reasons:expr) => {
        impl From<SdkError<$error>> for RaidenError {
            fn from(error: SdkError<$error>) -> Self {
                into_raiden_error(error, &[$($code),*], $reasons)
            }
        }
    };
}

impl_service_error!(
    BatchGetItemError,
    [
        "InternalServerError",
        "ProvisionedThroughputExceededException",
        "RequestLimitExceeded",
        "ResourceNotFoundException",
    ],
    |_| vec![]
);

impl_service_error!(
    BatchWriteItemError,
    [
        "InternalServerError",
        "ItemCollectionSizeLimitExceededException",
        "ProvisionedThroughputExceededException",
        "RequestLimitExceeded",
        "ResourceNotFoundException",
    ],
    |_| vec![]
);

impl_service_error!(
    GetItemError,
    [
        "InternalServerError",
        "ProvisionedThroughputExceededException",
        "RequestLimitExceeded",
        "ResourceNotFoundException",
    ],
    |_| vec![]
);

impl_service_error!(
    QueryError,
    [
        "InternalServerError",
        "ProvisionedThroughputExceededException",
        "RequestLimitExceeded",
        "ResourceNotFoundException",
    ],
    |_| vec![]
);

impl_service_error!(
    ScanError,
    [
        "InternalServerError",
        "ProvisionedThroughputExceededException",
        "RequestLimitExceeded",
        "ResourceNotFoundException",
    ],
    |_| vec![]
);

impl_service_error!(
    PutItemError,
    [
        "InternalServerError",
        "ProvisionedThroughputExceededException",
        "RequestLimitExceeded",
        "ResourceNotFoundException",
        "ConditionalCheckFailedException",
        "ItemCollectionSizeLimitExceededException",
        "TransactionConflictException",
    ],
    |_| vec![]
);

impl_service_error!(
    UpdateItemError,
    [
        "InternalServerError",
        "ProvisionedThroughputExceededException",
        "RequestLimitExceeded",
        "ResourceNotFoundException",
        "ConditionalCheckFailedException",
        "ItemCollectionSizeLimitExceededException",
        "TransactionConflictException",
    ],
    |_| vec![]
);

impl_service_error!(
    DeleteItemError,
    [
        "InternalServerError",
        "ProvisionedThroughputExceededException",
        "RequestLimitExceeded",
        "ResourceNotFoundException",
        "ConditionalCheckFailedException",
        "ItemCollectionSizeLimitExceededException",
        "TransactionConflictException",
    ],
    |_| vec![]
);

impl_service_error!(
    TransactWriteItemsError,
    [
        "IdempotentParameterMismatchException",
        "InternalServerError",
        "ProvisionedThroughputExceededException",
        "RequestLimitExceeded",
        "ResourceNotFoundException",
        "TransactionCanceledException",
        "TransactionInProgressException",
    ],
    |error| match error {
        TransactWriteItemsError::TransactionCanceledException(error) =>
            error.cancellation_reasons.clone().unwrap_or_default(),
        _ => vec![],
    }
);

impl_service_error!(
    TransactGetItemsError,
    [
        "InternalServerError",
        "ProvisionedThroughputExceededException",
        "RequestLimitExceeded",
        "ResourceNotFoundException",
        "TransactionCanceledException",
    ],
    |error| match error {
        TransactGetItemsError::TransactionCanceledException(error) =>
            error.cancellation_reasons.clone().unwrap_or_default(),
        _ => vec![],
    }
);
