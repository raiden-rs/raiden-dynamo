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
    #[raiden(sort_key)]
    pub year: usize,
    #[raiden(uuid)]
    pub uuid: String,
    pub name: String,
}

#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
async fn example() {
    let client = User::client(Region::Custom {
        endpoint: "http://localhost:8000".into(),
        name: "ap-northeast-1".into(),
    });
    //let user = UserPutItemInput {
    //    id: "a".to_owned(),
    //    name: "bokuweb".to_owned(),
    //    // uuid: "aa".to_owned(),
    //};
    //let cond = User::condition()
    //    .attr(User::name())
    //    .eq_attr(User::name());
    //
    //// let cond = User::condition().not().attr_type(User::name(), AttributeType::N);
    //// .and(User::condition().not().attribute_exists(User::id()));
    let keys: Vec<(&str, usize)> = vec![("bokuweb", 2019), ("raiden", 2020)];
    let _ = client.batch_get(keys).run().await;
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

    let client = User::client_with(sdk_client);
    let keys: Vec<(&str, usize)> = vec![("bokuweb", 2019), ("raiden", 2020)];
    let _ = client.batch_get(keys).run().await;
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
