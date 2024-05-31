use raiden::*;
use tracing_subscriber::{
    fmt::{format::FmtSpan, time::UtcTime},
    EnvFilter,
};

#[derive(Raiden, Debug)]
#[raiden(table_name = "hello")]
pub struct Hello {
    #[raiden(partition_key)]
    pub id: String,
    #[raiden(sort_key)]
    pub year: usize,
}

#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
async fn example() {
    let client = Hello::client(Region::Custom {
        endpoint: "http://localhost:8000".into(),
        name: "ap-northeast-1".into(),
    });
    //let user = HelloPutItemInput {
    //    id: "a".to_owned(),
    //    name: "bokuweb".to_owned(),
    //    // uuid: "aa".to_owned(),
    //};
    //let cond = Hello::condition()
    //    .attr(Hello::name())
    //    .eq_attr(Hello::name());
    //
    //// let cond = Hello::condition().not().attr_type(Hello::name(), AttributeType::N);
    //// .and(Hello::condition().not().attribute_exists(Hello::id()));
    let keys: Vec<(&str, usize)> = vec![("bokuweb", 2019), ("raiden", 2020)];
    let res = client.batch_get(keys).run().await;

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
    let client = Hello::client_with(sdk_client);
    let keys: Vec<(&str, usize)> = vec![("bokuweb", 2019), ("raiden", 2020)];
    let res = client.batch_get(keys).run().await;

    dbg!(&res);
    assert!(res.is_ok());
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("hello=debug,info"))
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(true)
        .with_timer(UtcTime::rfc_3339())
        .init();

    tokio::runtime::Runtime::new().unwrap().block_on(example());
}
