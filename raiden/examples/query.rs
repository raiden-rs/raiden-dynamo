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

#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
async fn example() {
    let client = QueryTestData0::client(Region::Custom {
        endpoint: "http://localhost:8000".into(),
        name: "ap-northeast-1".into(),
    });
    let cond = QueryTestData0::key_condition(QueryTestData0::id())
        .eq("id0")
        .and(QueryTestData0::key_condition(QueryTestData0::year()).eq(1999));
    let res = client.query().key_condition(cond).run().await;

    dbg!(&res);
    assert!(res.is_ok());

    let cond = QueryTestData0::key_condition(QueryTestData0::id())
        .eq("id0")
        .and(QueryTestData0::key_condition(QueryTestData0::year()).eq(1999));
    let res = client.query().key_condition(cond).run().await;

    dbg!(&res);
    assert!(res.is_ok());

    let cond = QueryTestData0::key_condition(QueryTestData0::id()).eq("id0");
    let filter = QueryTestData0::filter_expression(QueryTestData0::num()).eq(1000);
    let res = client
        .query()
        .key_condition(cond)
        .filter(filter)
        .run()
        .await;

    dbg!(&res);
    assert!(res.is_ok());
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
    let client = QueryTestData0::client_with(sdk_client);
    let cond = QueryTestData0::key_condition(QueryTestData0::id())
        .eq("id0")
        .and(QueryTestData0::key_condition(QueryTestData0::year()).eq(1999));
    let res = client.query().key_condition(cond).run().await;

    dbg!(&res);
    assert!(res.is_ok());

    let cond = QueryTestData0::key_condition(QueryTestData0::id())
        .eq("id0")
        .and(QueryTestData0::key_condition(QueryTestData0::year()).eq(1999));
    let res = client.query().key_condition(cond).run().await;

    dbg!(&res);
    assert!(res.is_ok());

    let cond = QueryTestData0::key_condition(QueryTestData0::id()).eq("id0");
    let filter = QueryTestData0::filter_expression(QueryTestData0::num()).eq(1000);
    let res = client
        .query()
        .key_condition(cond)
        .filter(filter)
        .run()
        .await;

    dbg!(&res);
    assert!(res.is_ok());
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("query=debug,info"))
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(true)
        .with_timer(UtcTime::rfc_3339())
        .init();

    tokio::runtime::Runtime::new().unwrap().block_on(example());
}
