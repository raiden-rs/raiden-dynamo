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
    pub name: String,
}

#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
async fn example() {
    let dispatcher =
        raiden::request::HttpClient::new().expect("failed to create request dispatcher");
    let credentials_provider = raiden::credential::DefaultCredentialsProvider::new()
        .expect("failed to create credentials provider");
    let core_client = raiden::Client::new_with(credentials_provider, dispatcher);
    let tx = ::raiden::WriteTx::new_with_client(
        core_client,
        Region::Custom {
            endpoint: "http://localhost:8000".into(),
            name: "ap-northeast-1".into(),
        },
    );
    let cond = User::condition().attr_not_exists(User::id());
    let input = User::put_item_builder()
        .id("testId".to_owned())
        .name("bokuweb".to_owned())
        .build();
    let input2 = User::put_item_builder()
        .id("testId2".to_owned())
        .name("bokuweb".to_owned())
        .build();
    let res = tx
        .put(User::put(input).condition(cond))
        .put(User::put(input2))
        .run()
        .await;

    dbg!(&res);
    assert!(res.is_ok());
}

#[cfg(feature = "aws-sdk")]
async fn example() {
    let https_connector = hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()
        .expect("should be success")
        .https_or_http()
        .enable_http1()
        .enable_http2()
        .build();
    let http_client = aws_smithy_runtime::client::http::hyper_014::HyperClientBuilder::new()
        .build(https_connector);
    let sdk_config = ::raiden::aws_sdk::aws_config::defaults(
        ::raiden::aws_sdk::config::BehaviorVersion::latest(),
    )
    .endpoint_url("http://localhost:8000")
    .http_client(http_client)
    .region(raiden::config::Region::from_static("ap-northeast-1"))
    .timeout_config(
        raiden::config::timeout::TimeoutConfig::builder()
            .connect_timeout(std::time::Duration::from_secs(5))
            .build(),
    )
    .load()
    .await;
    let sdk_client = ::raiden::Client::new(&sdk_config);
    let tx = ::raiden::WriteTx::new_with_client(sdk_client);
    let cond = User::condition().attr_not_exists(User::id());
    let input = User::put_item_builder()
        .id("testId".to_owned())
        .name("bokuweb".to_owned())
        .build();
    let input2 = User::put_item_builder()
        .id("testId2".to_owned())
        .name("bokuweb".to_owned())
        .build();
    let res = tx
        .put(User::put(input).condition(cond))
        .put(User::put(input2))
        .run()
        .await;

    dbg!(&res);
    assert!(res.is_ok());
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("raiden=debug,info"))
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(true)
        .with_timer(UtcTime::rfc_3339())
        .init();

    tokio::runtime::Runtime::new().unwrap().block_on(example());
}
