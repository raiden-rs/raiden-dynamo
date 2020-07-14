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
    #[error("`{0}`")]
    TransactionCanceled(String),
    #[error("`{0}`")]
    TransactionInProgress(String),
    #[error("`{0}`")]
    IdempotentParameterMismatch(String),
    #[error("blocking error")]
    Blocking,
    #[error("next_token decode error")]
    NextTokenDecodeError,
    #[error("attribute {attr_name:?} convert error")]
    AttributeConvertError { attr_name: String },
    #[error("attribute {attr_name:?} value not found")]
    AttributeValueNotFoundError { attr_name: String },
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
            RusotoError::HttpDispatch(e) => RaidenError::HttpDispatch(e),
            RusotoError::Credentials(e) => RaidenError::Credentials(e),
            RusotoError::Validation(msg) => RaidenError::Validation(msg),
            RusotoError::ParseError(msg) => RaidenError::ParseError(msg),
            RusotoError::Unknown(res) => RaidenError::Unknown(res),
            RusotoError::Blocking => RaidenError::Blocking,
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
            RusotoError::HttpDispatch(e) => RaidenError::HttpDispatch(e),
            RusotoError::Credentials(e) => RaidenError::Credentials(e),
            RusotoError::Validation(msg) => RaidenError::Validation(msg),
            RusotoError::ParseError(msg) => RaidenError::ParseError(msg),
            RusotoError::Unknown(res) => RaidenError::Unknown(res),
            RusotoError::Blocking => RaidenError::Blocking,
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
            RusotoError::HttpDispatch(e) => RaidenError::HttpDispatch(e),
            RusotoError::Credentials(e) => RaidenError::Credentials(e),
            RusotoError::Validation(msg) => RaidenError::Validation(msg),
            RusotoError::ParseError(msg) => RaidenError::ParseError(msg),
            RusotoError::Unknown(res) => RaidenError::Unknown(res),
            RusotoError::Blocking => RaidenError::Blocking,
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
                    RaidenError::TransactionCanceled(msg)
                }
                TransactWriteItemsError::TransactionInProgress(msg) => {
                    RaidenError::TransactionInProgress(msg)
                }
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
