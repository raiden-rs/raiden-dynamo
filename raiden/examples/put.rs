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

impl raiden::IntoAttribute for CustomId {
    fn into_attr(self) -> raiden::AttributeValue {
        raiden::AttributeValue {
            s: Some(self.0),
            ..::raiden::AttributeValue::default()
        }
    }
}

impl raiden::FromAttribute for CustomId {
    fn from_attr(value: Option<raiden::AttributeValue>) -> Result<Self, ConversionError> {
        Ok(CustomId(value.unwrap().s.unwrap()))
    }
}

#[derive(Raiden)]
#[raiden(table_name = "user")]
pub struct User {
    #[raiden(partition_key)]
    pub id: String,
    #[raiden(uuid)]
    pub uuid: CustomId,
    pub name: String,
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

    let rt = tokio::runtime::Runtime::new().unwrap();
    async fn example() {
        let client = User::client(Region::Custom {
            endpoint: "http://localhost:8000".into(),
            name: "ap-northeast-1".into(),
        });
        let input = User::put_item_builder()
            .id("testId".to_owned())
            .name("bokuweb".to_owned())
            .build();
        let _ = client.put(input).run().await;
    }
    rt.block_on(example());
}
