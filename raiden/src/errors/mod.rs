use crate::*;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum RaidenError {
    #[error("`{0}`")]
    ConditionCheckError(String),
    #[error("`{0}`")]
    DataExistsError(String),
    #[error("`{0}`")]
    ConditionalCheckFailed(String),
    #[error("`{0}`")]
    ItemCollectionSizeLimitExceeded(String),
    #[error("`{0}`")]
    TransactionConflict(String),
    #[error("`{0}`")]
    ResourceNotFound(String),
    #[error("`{0}`")]
    RequestLimitExceeded(String),
    #[error("`{0}`")]
    TransactionConflictError(String),
    #[error("`{0}`")]
    SizeLimitExceeded(String),
    #[error("`{0}`")]
    InternalServerError(String),
    #[error("`{0}`")]
    ProvisionedThroughputExceeded(String),
    #[error("`{0}`")]
    HttpDispatch(crate::request::HttpDispatchError),
    #[error("`{0}`")]
    Credentials(crate::CredentialsError),
    #[error("`{0}`")]
    Validation(String),
    #[error("`{0}`")]
    ParseError(String),
    #[error("unknown error")]
    Unknown(crate::request::BufferedHttpResponse),
    #[error("blocking error")]
    Blocking,
    #[error("next_token decode error")]
    NextTokenDecodeError,
    #[error("attribute {attr_name:?} convert error")]
    AttributeConvertError { attr_name: String },
    #[error("attribute {attr_name:?} value not found")]
    AttributeValueNotFoundError { attr_name: String },
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
            RusotoError::HttpDispatch(e) => RaidenError::HttpDispatch(e),
            RusotoError::Credentials(e) => RaidenError::Credentials(e),
            RusotoError::Validation(msg) => RaidenError::Validation(msg),
            RusotoError::ParseError(msg) => RaidenError::ParseError(msg),
            RusotoError::Unknown(res) => RaidenError::Unknown(res),
            RusotoError::Blocking => RaidenError::Blocking,
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
            RusotoError::HttpDispatch(e) => RaidenError::HttpDispatch(e),
            RusotoError::Credentials(e) => RaidenError::Credentials(e),
            RusotoError::Validation(msg) => RaidenError::Validation(msg),
            RusotoError::ParseError(msg) => RaidenError::ParseError(msg),
            RusotoError::Unknown(res) => RaidenError::Unknown(res),
            RusotoError::Blocking => RaidenError::Blocking,
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
            RusotoError::HttpDispatch(e) => RaidenError::HttpDispatch(e),
            RusotoError::Credentials(e) => RaidenError::Credentials(e),
            RusotoError::Validation(msg) => RaidenError::Validation(msg),
            RusotoError::ParseError(msg) => RaidenError::ParseError(msg),
            RusotoError::Unknown(res) => RaidenError::Unknown(res),
            RusotoError::Blocking => RaidenError::Blocking,
        }
    }
}
