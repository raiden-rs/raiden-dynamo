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

#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
async fn example() {
    let client = Reserved::client(Region::Custom {
        endpoint: "http://localhost:8000".into(),
        name: "ap-northeast-1".into(),
    });

    let _ = client.get("id0").run().await;
}

#[cfg(feature = "aws-sdk")]
async fn example() {
    let sdk_config = aws_config::SdkConfig::builder()
        .endpoint_url("http://localhost:8000")
        .region(raiden::Region::from_static("ap-northeast-1"))
        .credentials_provider(
            aws_credential_types::provider::SharedCredentialsProvider::new(
                aws_credential_types::Credentials::new("dummy", "dummy", None, None, "dummy"),
            ),
        )
        .build();
    let sdk_client = aws_sdk_dynamodb::Client::new(&sdk_config);

    let client = Reserved::client_with(sdk_client);
    let _ = client.get("id0").run().await;
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

    tokio::runtime::Runtime::new().unwrap().block_on(example());
}
