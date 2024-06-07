use raiden::*;
use tracing_subscriber::{
    fmt::{format::FmtSpan, time::UtcTime},
    EnvFilter,
};

#[derive(Raiden, Debug)]
#[raiden(table_name = "LastEvaluateKeyData")]
pub struct Test {
    #[raiden(partition_key)]
    pub id: String,
    pub ref_id: String,
    pub long_text: LongText,
}

#[derive(Clone, PartialEq)]
pub struct LongText(String);

#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
impl IntoAttribute for LongText {
    fn into_attr(self) -> AttributeValue {
        AttributeValue {
            s: Some(self.0),
            ..AttributeValue::default()
        }
    }
}

#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
impl FromAttribute for LongText {
    fn from_attr(value: Option<AttributeValue>) -> Result<Self, ConversionError> {
        Ok(Self(value.unwrap().s.unwrap()))
    }
}

#[cfg(feature = "aws-sdk")]
impl raiden::IntoAttribute for LongText {
    fn into_attr(self) -> raiden::AttributeValue {
        raiden::AttributeValue::S(self.0)
    }
}

#[cfg(feature = "aws-sdk")]
impl raiden::FromAttribute for LongText {
    fn from_attr(value: Option<raiden::AttributeValue>) -> Result<Self, ConversionError> {
        if let Some(raiden::AttributeValue::S(v)) = value {
            Ok(Self(v))
        } else {
            unimplemented!();
        }
    }
}

impl std::fmt::Debug for LongText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Long long text")
    }
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

    let client = Test::client_with(sdk_client);
    let cond = Test::key_condition(Test::ref_id()).eq("id0");
    let res = client
        .query()
        .index("testGSI")
        .limit(5)
        .key_condition(cond)
        .run()
        .await;

    dbg!(&res);
    assert!(res.is_ok());
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
