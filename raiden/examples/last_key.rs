use raiden::*;
use tracing_subscriber::{
    fmt::{format::FmtSpan, time::UtcTime},
    EnvFilter,
};

#[derive(Raiden)]
#[raiden(table_name = "LastEvaluateKeyData")]
pub struct Test {
    #[raiden(partition_key)]
    pub id: String,
    pub ref_id: String,
    pub long_text: String,
}

#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
async fn example() {
    let client = Test::client(Region::Custom {
        endpoint: "http://localhost:8000".into(),
        name: "ap-northeast-1".into(),
    });
    let cond = Test::key_condition(Test::ref_id()).eq("id0");

    let res = client
        .query()
        .index("testGSI")
        .limit(5)
        .key_condition(cond)
        .run()
        .await;
    dbg!(&res.unwrap().items.len());
}

#[cfg(feature = "aws-sdk")]
async fn example() {
    let sdk_config = raiden::AwsSdkConfig::builder()
        .behavior_version(raiden::BehaviorVersion::latest())
        .credentials_provider(
            aws_credential_types::provider::SharedCredentialsProvider::new(
                aws_credential_types::Credentials::new("dummy", "dummy", None, None, "dummy"),
            ),
        )
        .endpoint_url("http://localhost:8000")
        .region(raiden::Region::from_static("ap-northeast-1"))
        .build();
    let sdk_client = aws_sdk_dynamodb::Client::new(&sdk_config);

    let client = Test::client_with(sdk_client);
    let cond = Test::key_condition(Test::ref_id()).eq("id0");
    let res = client
        .query()
        .index("testGSI")
        .limit(5)
        .key_condition(cond)
        .run()
        .await;
    dbg!(&res.unwrap().items.len());
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("last_key=debug,info"))
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(true)
        .with_timer(UtcTime::rfc_3339())
        .init();

    tokio::runtime::Runtime::new().unwrap().block_on(example());
}
