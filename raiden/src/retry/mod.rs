pub use again::{Condition, RetryPolicy};
use std::sync::atomic::AtomicUsize;
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
    pub count: std::sync::Arc<AtomicUsize>,
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
            count: std::sync::Arc::new(AtomicUsize::new(0)),
            strategy: Box::new(DefaultRetryStrategy),
        }
    }
}

impl Condition<super::RaidenError> for &RetryCondition {
    fn is_retryable(&mut self, error: &RaidenError) -> bool {
        use std::sync::atomic::Ordering;
        let count = self.count.load(Ordering::Relaxed);
        let retryable = self.strategy.should_retry(error, count);
        self.count.store(count.wrapping_add(1), Ordering::Relaxed);
        retryable
    }
}

pub trait RetryStrategy {
    fn should_retry(&self, error: &RaidenError, count: usize) -> bool;
    fn policy(&self) -> Policy;
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct DefaultRetryStrategy;

impl RetryStrategy for DefaultRetryStrategy {
    fn should_retry(&self, error: &RaidenError, count: usize) -> bool {
        log::debug!("request count is {}", count);
        matches!(
            error,
            RaidenError::InternalServerError(_)
                | RaidenError::ProvisionedThroughputExceeded(_)
                | RaidenError::RequestLimitExceeded(_)
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
