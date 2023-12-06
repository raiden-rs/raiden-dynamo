use raiden::*;
use tracing_subscriber::{
    fmt::{format::FmtSpan, time::UtcTime},
    EnvFilter,
};

#[derive(Raiden)]
#[raiden(table_name = "user")]
pub struct User {
    #[raiden(partition_key)]
    pub id: String,
}

struct MyRetryStrategy;

impl RetryStrategy for MyRetryStrategy {
    fn should_retry(&self, _error: &RaidenError) -> bool {
        true
    }

    fn policy(&self) -> Policy {
        Policy::Limit(3)
    }
}

#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
async fn example() {
    let client = User::client(Region::Custom {
        endpoint: "http://localhost:8000".into(),
        name: "ap-northeast-1".into(),
    });

    let _ = client
        .with_retries(Box::new(MyRetryStrategy))
        .get("anonymous")
        .run()
        .await;
}

#[cfg(feature = "aws-sdk")]
async fn example() {
    let sdk_config = raiden::config::defaults(raiden::BehaviorVersion::latest())
        .endpoint_url("http://localhost:8000")
        .region(raiden::Region::from_static("ap-northeast-1"))
        .load()
        .await;
    let sdk_client = aws_sdk_dynamodb::Client::new(&sdk_config);

    let client = User::client_with(sdk_client);
    let _ = client
        .with_retries(Box::new(MyRetryStrategy))
        .get("anonymous")
        .run()
        .await;
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("get_with_retries=debug,info"))
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(true)
        .with_timer(UtcTime::rfc_3339())
        .init();

    tokio::runtime::Runtime::new().unwrap().block_on(example());
}
