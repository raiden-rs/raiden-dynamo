pub use again::{Condition, RetryPolicy};
use std::time::Duration;

use super::RaidenError;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Policy {
    None,
    Limit(usize),
    Pause(usize, Duration),
    Exponential(usize, Duration),
}

impl Default for Policy {
    fn default() -> Self {
        Policy::Exponential(5, Duration::from_millis(20))
    }
}

#[allow(clippy::from_over_into)]
impl Into<RetryPolicy> for Policy {
    fn into(self) -> RetryPolicy {
        match self {
            Policy::None => RetryPolicy::default().with_max_retries(0),
            Policy::Limit(times) => RetryPolicy::default()
                .with_max_retries(times)
                .with_jitter(true),
            Policy::Pause(times, duration) => RetryPolicy::fixed(duration)
                .with_max_retries(times)
                .with_jitter(true),
            Policy::Exponential(times, duration) => RetryPolicy::exponential(duration)
                .with_max_retries(times)
                .with_jitter(true),
        }
    }
}

pub struct RetryCondition {
    pub strategy: Box<dyn RetryStrategy + Send + Sync>,
}

impl RetryCondition {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for RetryCondition {
    fn default() -> Self {
        Self {
            strategy: Box::new(DefaultRetryStrategy),
        }
    }
}

impl Condition<super::RaidenError> for &RetryCondition {
    fn is_retryable(&mut self, error: &RaidenError) -> bool {
        self.strategy.should_retry(error)
    }
}

pub trait RetryStrategy {
    fn should_retry(&self, error: &RaidenError) -> bool;
    fn policy(&self) -> Policy;
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct DefaultRetryStrategy;

impl RetryStrategy for DefaultRetryStrategy {
    fn should_retry(&self, error: &RaidenError) -> bool {
        matches!(
            error,
            RaidenError::InternalServerError(_)
                | RaidenError::ProvisionedThroughputExceeded(_)
                | RaidenError::RequestLimitExceeded(_)
                // Sometimes I ran into `HttpDispatchError { message: "Error during dispatch: connection closed before message completed" }` and
                // CredentialsError { message: "Request ID: Some(\"xxx\") Body: <ErrorResponse xmlns=\"https://sts.amazonaws.com/doc/2011-06-15/\">\n  <Error>\n    <Type>Sender</Type>\n    <Code>Throttling</Code>\n    <Message>Rate exceeded</Message>\n  </Error>\n  <RequestId>xxx</RequestId>\n</ErrorResponse>\n" }
                | RaidenError::HttpDispatch(_)
                | RaidenError::Credentials(_)
                // INFO: For now, return true, when unknown error detected.
                //       This is because, sometimes throttlingException is included in unknown error.
                //       please make more rigorous classification of errors.
                | RaidenError::Unknown(_)
        )
    }

    fn policy(&self) -> Policy {
        Policy::default()
    }
}
