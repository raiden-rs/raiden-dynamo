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
    tx.put(User::put(input).condition(cond))
        .put(User::put(input2))
        .run()
        .await
        .unwrap();
}

#[cfg(feature = "aws-sdk")]
async fn example() {
    use aws_smithy_client::{http_connector::ConnectorSettings, hyper_ext};

    let https_connector = hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()
        .https_or_http()
        .enable_http1()
        .enable_http2()
        .build();
    let smithy_connector = hyper_ext::Adapter::builder()
        .connector_settings(
            ConnectorSettings::builder()
                .connect_timeout(std::time::Duration::from_secs(5))
                .build(),
        )
        .build(https_connector);

    let sdk_config = aws_config::SdkConfig::builder()
        .endpoint_url("http://localhost:8000")
        .region(raiden::Region::from_static("ap-northeast-1"))
        .credentials_provider(
            aws_credential_types::provider::SharedCredentialsProvider::new(
                aws_credential_types::Credentials::new("dummy", "dummy", None, None, "dummy"),
            ),
        )
        .http_connector(smithy_connector)
        .build();
    let sdk_client = aws_sdk_dynamodb::Client::new(&sdk_config);

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
    tx.put(User::put(input).condition(cond))
        .put(User::put(input2))
        .run()
        .await
        .unwrap();
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
