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

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("delete=debug,info"))
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(true)
        .with_timer(UtcTime::rfc_3339())
        .init();

    let rt = tokio::runtime::Runtime::new().unwrap();
    async fn example() {
        let client = Test::client(Region::Custom {
            endpoint: "http://localhost:8000".into(),
            name: "ap-northeast-1".into(),
        });

        let res = client.delete("id1", 2003_usize).run().await;
        dbg!(&res);
    }
    rt.block_on(example());
}
