//! Compiles the rustls Rusoto backend together with the builder patterns
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

pub fn http_client() -> rusoto_core::HttpClient {
    rusoto_core::HttpClient::new().expect("rustls HttpClient")
}
