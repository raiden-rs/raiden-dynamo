pub use again::{Condition, RetryPolicy};
use std::time::Duration;

use super::RaidenError;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Policy {
    Limit(usize),
    Pause(usize, Duration),
    Exponential(usize, Duration),
}

impl Default for Policy {
    fn default() -> Self {
        Policy::Exponential(5, Duration::from_millis(100))
    }
}

impl Into<RetryPolicy> for Policy {
    fn into(self) -> RetryPolicy {
        match self {
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

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct DefaultRetryCondition;

impl Condition<super::RaidenError> for DefaultRetryCondition {
    fn is_retryable(&mut self, error: &RaidenError) -> bool {
        matches!(error, RaidenError::InternalServerError(_) | RaidenError::ProvisionedThroughputExceeded(_) | RaidenError::RequestLimitExceeded(_))
    }
}
