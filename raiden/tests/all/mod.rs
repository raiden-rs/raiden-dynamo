mod batch_delete;
mod batch_get;
mod condition;
mod delete;
mod filter_expression;
mod get;
mod key_condition;
mod put;
mod query;
mod rename;
mod rename_all;
mod scan;
mod transact_write;
mod update;

#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
macro_rules! create_client {
    ($ty: ty) => {
        <$ty>::new(raiden::Region::Custom {
            endpoint: "http://localhost:8000".into(),
            name: "ap-northeast-1".into(),
        })
    };
}

#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
macro_rules! create_client_from_struct {
    ($ty: ty) => {
        <$ty>::client(raiden::Region::Custom {
            endpoint: "http://localhost:8000".into(),
            name: "ap-northeast-1".into(),
        })
    };
}

#[cfg(feature = "aws-sdk")]
macro_rules! create_client {
    ($ty: ty) => {{
        let sdk_config = ::raiden::aws_sdk::aws_config::defaults(
            ::raiden::aws_sdk::config::BehaviorVersion::latest(),
        )
        .endpoint_url("http://localhost:8000")
        .region(::raiden::config::Region::from_static("ap-northeast-1"))
        .load()
        .await;
        let sdk_client = ::raiden::Client::new(&sdk_config);

        <$ty>::new_with_client(sdk_client)
    }};
}

#[cfg(feature = "aws-sdk")]
macro_rules! create_client_from_struct {
    ($ty: ty) => {{
        let sdk_config = ::raiden::aws_sdk::aws_config::defaults(
            ::raiden::aws_sdk::config::BehaviorVersion::latest(),
        )
        .endpoint_url("http://localhost:8000")
        .region(::raiden::config::Region::from_static("ap-northeast-1"))
        .load()
        .await;
        let sdk_client = ::raiden::Client::new(&sdk_config);

        <$ty>::client_with(sdk_client)
    }};
}

use {create_client, create_client_from_struct};

#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
fn default_key_and_attributes() -> raiden::KeysAndAttributes {
    raiden::KeysAndAttributes {
        attributes_to_get: None,
        consistent_read: None,
        expression_attribute_names: None,
        keys: vec![],
        projection_expression: None,
    }
}

#[cfg(feature = "aws-sdk")]
fn default_key_and_attributes() -> raiden::aws_sdk::types::KeysAndAttributes {
    let mut v = raiden::aws_sdk::types::KeysAndAttributes::builder()
        .keys(std::collections::HashMap::new())
        .build()
        .expect("should be built");

    v.keys = vec![];
    v
}
