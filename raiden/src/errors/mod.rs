mod transaction;

pub use transaction::*;

#[derive(thiserror::Error, Debug)]
pub enum RaidenError {
    #[error("attribute {attr_name:?} convert error")]
    AttributeConvertError { attr_name: String },
    #[error("`{0}`")]
    ConditionalCheckFailed(String),
    #[error("`{0}`")]
    IdempotentParameterMismatch(String),
    #[error("`{0}`")]
    InternalServerError(String),
    #[error("`{0}`")]
    ItemCollectionSizeLimitExceeded(String),
    #[error("next_token decode error")]
    NextTokenDecodeError,
    #[error("`{0}`")]
    ProvisionedThroughputExceeded(String),
    #[error("`{0}`")]
    RequestLimitExceeded(String),
    #[error("`{0}`")]
    ResourceNotFound(String),
    #[error("`{0}`")]
    SizeLimitExceeded(String),
    #[error("`{0}`")]
    TransactionConflict(String),
    #[error("`{0}`")]
    TransactionInProgress(String),
    //
    // Next errors returns only using rusoto.
    //
    #[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
    #[error("blocking error")]
    Blocking,
    #[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
    #[error("`{0}`")]
    Credentials(crate::CredentialsError),
    #[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
    #[error("`{0}`")]
    HttpDispatch(crate::HttpDispatchError),
    #[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
    #[error("`{0}`")]
    ParseError(String),
    #[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
    #[error("`transaction canceled error {reasons}`")]
    TransactionCanceled {
        reasons: RaidenTransactionCancellationReasons,
    },
    #[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
    #[error("unknown error")]
    Unknown(crate::request::BufferedHttpResponse),
    #[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
    #[error("`{0}`")]
    Validation(String),
    //
    // Next errors returns only using aws-sdk.
    //
    #[cfg(feature = "aws-sdk")]
    #[error("`{0:?}`")]
    Construction(aws_smithy_http::result::ConstructionFailure),
    #[cfg(feature = "aws-sdk")]
    #[error("`{0:?}`")]
    HttpDispatch(aws_smithy_http::result::DispatchFailure),
    #[cfg(feature = "aws-sdk")]
    #[error("`{0:?}`")]
    Timeout(aws_smithy_http::result::TimeoutError),
    #[cfg(feature = "aws-sdk")]
    #[error("`transaction canceled error {reasons}: {raw_reasons:?}`")]
    TransactionCanceled {
        reasons: RaidenTransactionCancellationReasons,
        raw_reasons: Vec<crate::CancellationReason>,
    },
    #[cfg(feature = "aws-sdk")]
    #[error("unknown error")]
    Unknown(aws_smithy_runtime_api::client::orchestrator::HttpResponse),
    //
    // Next errors are not used.
    //
    #[deprecated = "unused. this variant never returns."]
    #[error("attribute {attr_name:?} value not found")]
    AttributeValueNotFoundError { attr_name: String },
    #[deprecated = "unused. this variant never returns."]
    #[error("`{0}`")]
    DataExistsError(String),
    #[deprecated = "unused. this variant never returns."]
    #[error("`{0}`")]
    TransactionConflictError(String),
}
