use raiden::*;
use tracing_subscriber::{
    fmt::{format::FmtSpan, time::UtcTime},
    EnvFilter,
};

#[derive(Raiden, Debug)]
#[raiden(table_name = "ScanWithFilterTestData0")]
#[allow(dead_code)]
pub struct Scan {
    #[raiden(partition_key)]
    id: String,
    name: String,
    year: usize,
    num: usize,
}

#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
async fn example() {
    let client = Scan::client(Region::Custom {
        endpoint: "http://localhost:8000".into(),
        name: "ap-northeast-1".into(),
    });
    let filter = Scan::filter_expression(Scan::num()).eq(1000);
    let res = client.scan().filter(filter).run().await;

    dbg!(&res);
    assert!(res.is_ok());
    assert_eq!(res.unwrap().items.len(), 50);
}

#[cfg(feature = "aws-sdk")]
async fn example() {
    let sdk_config = ::raiden::aws_sdk::aws_config::defaults(
        ::raiden::aws_sdk::config::BehaviorVersion::latest(),
    )
    .endpoint_url("http://localhost:8000")
    .region(::raiden::config::Region::from_static("ap-northeast-1"))
    .load()
    .await;
    let sdk_client = ::raiden::Client::new(&sdk_config);
    let client = Scan::client_with(sdk_client);
    let filter = Scan::filter_expression(Scan::num()).eq(1000);
    let res = client.scan().filter(filter).run().await;

    dbg!(&res);
    assert!(res.is_ok());
    assert_eq!(res.unwrap().items.len(), 50);
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("scan_with_filter=debug,info"))
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(true)
        .with_timer(UtcTime::rfc_3339())
        .init();

    tokio::runtime::Runtime::new().unwrap().block_on(example());
}
