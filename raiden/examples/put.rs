use raiden::*;
use tracing_subscriber::{
    fmt::{format::FmtSpan, time::UtcTime},
    EnvFilter,
};

#[derive(Debug, Clone, PartialEq)]
pub struct CustomId(String);

impl From<String> for CustomId {
    fn from(v: String) -> CustomId {
        CustomId(v)
    }
}

#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
impl raiden::IntoAttribute for CustomId {
    fn into_attr(self) -> raiden::AttributeValue {
        raiden::AttributeValue {
            s: Some(self.0),
            ..::raiden::AttributeValue::default()
        }
    }
}

#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
impl raiden::FromAttribute for CustomId {
    fn from_attr(value: Option<raiden::AttributeValue>) -> Result<Self, ConversionError> {
        Ok(CustomId(value.unwrap().s.unwrap()))
    }
}

#[cfg(feature = "aws-sdk")]
impl raiden::IntoAttribute for CustomId {
    fn into_attr(self) -> raiden::AttributeValue {
        raiden::AttributeValue::S(self.0)
    }
}

#[cfg(feature = "aws-sdk")]
impl raiden::FromAttribute for CustomId {
    fn from_attr(value: Option<raiden::AttributeValue>) -> Result<Self, ConversionError> {
        if let Some(raiden::AttributeValue::S(v)) = value {
            Ok(CustomId(v))
        } else {
            unimplemented!();
        }
    }
}

#[derive(Raiden, Debug)]
#[raiden(table_name = "user")]
pub struct User {
    #[raiden(partition_key)]
    pub id: String,
    #[raiden(uuid)]
    pub uuid: CustomId,
    pub name: String,
}

#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
async fn example() {
    let client = User::client(Region::Custom {
        endpoint: "http://localhost:8000".into(),
        name: "ap-northeast-1".into(),
    });
    let input = User::put_item_builder()
        .id("testId".to_owned())
        .name("bokuweb".to_owned())
        .build();
    let res = client.put(input).run().await;

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
    let client = User::client_with(sdk_client);
    let input = User::put_item_builder()
        .id("testId".to_owned())
        .name("bokuweb".to_owned())
        .build();
    let res = client.put(input).run().await;

    dbg!(&res);
    assert!(res.is_ok());
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("put=debug,info"))
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(true)
        .with_timer(UtcTime::rfc_3339())
        .init();

    tokio::runtime::Runtime::new().unwrap().block_on(example());
}
