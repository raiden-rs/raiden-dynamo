use again::Condition;
pub use again::RetryPolicy;
use rand::{distributions::OpenClosed01, thread_rng, Rng};
use std::future::Future;
use std::time::Duration;
use wasm_timer::Delay;

use super::RaidenError;

/// Retries the unprocessed items returned by a batch write operation.
///
/// DynamoDB returns throttled batch items as a successful response, so the
/// regular error retry policy does not apply to them. This helper gives that
/// response path the same bounded exponential backoff and jitter behavior.
#[doc(hidden)]
pub async fn retry_batch_write_unprocessed_items<T, E, F, Fut>(
    mut pending: Vec<T>,
    max_retries: usize,
    initial_backoff: Duration,
    mut task: F,
) -> Result<Vec<T>, E>
where
    F: FnMut(Vec<T>) -> Fut,
    Fut: Future<Output = Result<Vec<T>, E>>,
{
    for retry in 0..=max_retries {
        pending = task(pending).await?;
        if pending.is_empty() || retry == max_retries {
            return Ok(pending);
        }

        let factor = 1_u32.checked_shl(retry as u32).unwrap_or(u32::MAX);
        let maximum = initial_backoff.checked_mul(factor).unwrap_or(Duration::MAX);
        let jitter: f64 = thread_rng().sample(OpenClosed01);
        let delay = Duration::from_secs_f64(maximum.as_secs_f64() * jitter);
        let _ = Delay::new(delay).await;
    }

    unreachable!("the retry loop always returns")
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Policy {
    None,
    Limit(usize),
    Pause(usize, Duration),
    Exponential(usize, Duration),
}

impl Default for Policy {
    fn default() -> Self {
        Policy::Exponential(5, Duration::from_millis(50))
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

    pub fn never() -> Self {
        Self {
            strategy: Box::new(NopRetryStrategy),
        }
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
        #[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
        return matches!(
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
        );

        #[cfg(feature = "aws-sdk")]
        return matches!(
            error,
            RaidenError::InternalServerError(_)
                | RaidenError::ProvisionedThroughputExceeded(_)
                | RaidenError::RequestLimitExceeded(_)
                | RaidenError::HttpDispatch(_)
                | RaidenError::Timeout(_)
                // INFO: For now, return true, when unknown error detected.
                //       This is because, sometimes throttlingException is included in unknown error.
                //       please make more rigorous classification of errors.
                | RaidenError::Unknown(_)
        );
    }

    fn policy(&self) -> Policy {
        Policy::default()
    }
}

#[cfg(test)]
mod tests {
    use super::retry_batch_write_unprocessed_items;
    use std::{
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        },
        time::Duration,
    };

    #[tokio::test]
    async fn retries_unprocessed_items_until_they_are_empty() {
        let calls = Arc::new(AtomicUsize::new(0));
        let result = retry_batch_write_unprocessed_items(vec![1], 5, Duration::ZERO, |items| {
            let calls = Arc::clone(&calls);
            async move {
                let call = calls.fetch_add(1, Ordering::SeqCst);
                Ok::<_, ()>(if call < 2 { items } else { vec![] })
            }
        })
        .await
        .unwrap();

        assert!(result.is_empty());
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn returns_unprocessed_items_after_the_retry_limit() {
        let calls = Arc::new(AtomicUsize::new(0));
        let result = retry_batch_write_unprocessed_items(vec![1, 2], 2, Duration::ZERO, |items| {
            let calls = Arc::clone(&calls);
            async move {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok::<_, ()>(items)
            }
        })
        .await
        .unwrap();

        assert_eq!(result, vec![1, 2]);
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn returns_request_errors_without_retrying_them() {
        let calls = Arc::new(AtomicUsize::new(0));
        let result = retry_batch_write_unprocessed_items(vec![1], 5, Duration::ZERO, |_items| {
            let calls = Arc::clone(&calls);
            async move {
                calls.fetch_add(1, Ordering::SeqCst);
                Err::<Vec<i32>, _>("request failed")
            }
        })
        .await;

        assert_eq!(result, Err("request failed"));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct NopRetryStrategy;

impl RetryStrategy for NopRetryStrategy {
    fn should_retry(&self, _: &RaidenError) -> bool {
        false
    }

    fn policy(&self) -> Policy {
        Policy::None
    }
}
