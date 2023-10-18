use raiden::*;
use tracing_subscriber::{
    fmt::{format::FmtSpan, time::UtcTime},
    EnvFilter,
};

#[derive(Raiden)]
#[raiden(table_name = "ReservedTestData0")]
pub struct Reserved {
    #[raiden(partition_key)]
    pub id: String,
    pub r#type: String,
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("get_with_reserved=debug,info"))
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(true)
        .with_timer(UtcTime::rfc_3339())
        .init();

    let rt = tokio::runtime::Runtime::new().unwrap();
    async fn example() {
        let client = Reserved::client(Region::Custom {
            endpoint: "http://localhost:8000".into(),
            name: "ap-northeast-1".into(),
        });
        let _ = client.get("id0").run().await;
    }
    rt.block_on(example());
}
