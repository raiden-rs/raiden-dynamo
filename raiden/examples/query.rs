use raiden::*;
use tracing_subscriber::{
    fmt::{format::FmtSpan, time::UtcTime},
    EnvFilter,
};

#[derive(Raiden, Debug)]
#[allow(dead_code)]
pub struct QueryTestData0 {
    #[raiden(partition_key)]
    #[allow(dead_code)]
    id: String,
    name: String,
    year: usize,
    num: usize,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("query=debug,info"))
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(true)
        .with_timer(UtcTime::rfc_3339())
        .init();

    let client = QueryTestData0::client(Region::Custom {
        endpoint: "http://localhost:8000".into(),
        name: "ap-northeast-1".into(),
    });
    let cond = QueryTestData0::key_condition(QueryTestData0::id())
        .eq("id0")
        .and(QueryTestData0::key_condition(QueryTestData0::year()).eq(1999));
    let res = client.query().key_condition(cond).run().await;
    dbg!(&res);

    let cond = QueryTestData0::key_condition(QueryTestData0::id())
        .eq("id0")
        .and(QueryTestData0::key_condition(QueryTestData0::year()).eq(1999));
    let res = client.query().key_condition(cond).run().await;
    dbg!(&res);

    let cond = QueryTestData0::key_condition(QueryTestData0::id()).eq("id0");
    let filter = QueryTestData0::filter_expression(QueryTestData0::num()).eq(1000);
    let res = client
        .query()
        .key_condition(cond)
        .filter(filter)
        .run()
        .await;
    dbg!(&res);
}
