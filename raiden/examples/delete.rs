use raiden::*;
use tracing_subscriber::{
    fmt::{format::FmtSpan, time::UtcTime},
    EnvFilter,
};

#[derive(Raiden)]
#[raiden(table_name = "QueryTestData0")]
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Test {
    #[raiden(partition_key)]
    id: String,
    name: String,
    #[raiden(sort_key)]
    year: usize,
}

#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
async fn example() {
    let client = Test::client(raiden::Region::Custom {
        endpoint: "http://localhost:8000".into(),
        name: "ap-northeast-1".into(),
    });

    let res = client.delete("id1", 2003_usize).run().await;
    dbg!(&res);
}

#[cfg(feature = "aws-sdk")]
async fn example() {
    let sdk_config = raiden::config::defaults(raiden::BehaviorVersion::latest())
        .endpoint_url("http://localhost:8000")
        .region(raiden::Region::from_static("ap-northeast-1"))
        .load()
        .await;
    let sdk_client = aws_sdk_dynamodb::Client::new(&sdk_config);

    let client = Test::client_with(sdk_client);
    let res = client.delete("id1", 2003_usize).run().await;
    dbg!(&res);
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("delete=debug,info"))
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(true)
        .with_timer(UtcTime::rfc_3339())
        .init();

    tokio::runtime::Runtime::new().unwrap().block_on(example());
}
