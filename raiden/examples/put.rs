use raiden::*;

#[derive(Debug, Clone, PartialEq)]
pub struct CustomId(String);

impl Into<CustomId> for String {
    fn into(self) -> CustomId {
        CustomId(self)
    }
}

impl raiden::IntoAttribute for CustomId {
    fn into_attr(self: Self) -> raiden::AttributeValue {
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
    id: String,
    #[raiden(uuid)]
    uuid: CustomId,
    name: String,
}

fn main() {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
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
        dbg!(res);
    }
    rt.block_on(example());
}
