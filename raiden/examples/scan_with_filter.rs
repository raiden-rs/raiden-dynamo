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

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("scan_with_filter=debug,info"))
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(true)
        .with_timer(UtcTime::rfc_3339())
        .init();

    let client = Scan::client(Region::Custom {
        endpoint: "http://localhost:8000".into(),
        name: "ap-northeast-1".into(),
    });
    let filter = Scan::filter_expression(Scan::num()).eq(1000);
    let res = client.scan().filter(filter).run().await.unwrap();
    assert_eq!(res.items.len(), 50);
}
