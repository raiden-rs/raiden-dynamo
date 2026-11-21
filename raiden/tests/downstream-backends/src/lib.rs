//! Compiles both supported backends together with the builder patterns
//! downstream crates wrote against raiden 0.0.94.

use raiden::*;

#[derive(Raiden, Debug, Clone)]
#[raiden(table_name = "user")]
pub struct User {
    #[raiden(partition_key)]
    pub id: String,
    pub name: String,
    pub nickname: Option<String>,
}

#[cfg(feature = "rusoto")]
pub fn client() -> UserClient {
    User::client(Region::Custom {
        endpoint: "http://localhost:8000".into(),
        name: "ap-northeast-1".into(),
    })
}

pub fn start() -> DefaultUserPutItemInputBuilder {
    User::put_item_builder()
}

pub fn input(nickname: Option<String>) -> UserPutItemInput {
    let mut builder = start().id("id0".to_owned()).name("bokuweb".to_owned());
    if let Some(nickname) = nickname {
        builder = builder.nickname(nickname);
    }
    builder.build()
}

#[derive(Clone, Debug, Builder)]
#[builder(setter(into))]
pub struct Bookmark {
    pub id: String,
}

pub fn bookmark() -> Bookmark {
    Bookmark::builder().id("b0".to_owned()).build()
}

#[cfg(feature = "rusoto")]
pub fn http_client() -> rusoto_core::HttpClient {
    rusoto_core::HttpClient::new().expect("native-tls HttpClient")
}

// The replacement for Rusoto's retired rustls configuration: construct the AWS
// SDK client with its modern rustls connector, then pass it to the derived client.
#[cfg(feature = "aws-sdk")]
pub async fn client() -> UserClient {
    let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(aws_sdk_dynamodb::config::Region::new("ap-northeast-1"))
        .endpoint_url("http://localhost:8000")
        .http_client(
            aws_smithy_http_client::Builder::new()
                .tls_provider(aws_smithy_http_client::tls::Provider::Rustls(
                    aws_smithy_http_client::tls::rustls_provider::CryptoMode::AwsLc,
                ))
                .build_https(),
        )
        .load()
        .await;
    User::client_with(aws_sdk_dynamodb::Client::new(&config))
}
