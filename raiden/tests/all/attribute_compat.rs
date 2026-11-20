//! Real DynamoDB validation and read-after-write regressions shared by both backends.
use raiden::*;

#[derive(Raiden, Debug, Clone, PartialEq)]
#[raiden(table_name = "user")]
pub struct Settings {
    #[raiden(partition_key)]
    pub id: String,
    our_names: Document<Vec<String>>,
}

fn settings(id: &str, names: Vec<String>) -> SettingsPutItemInput {
    SettingsPutItemInput {
        id: id.into(),
        our_names: Document::new(names),
    }
}

#[tokio::test]
async fn document_list_survives_zero_one_zero_and_all_write_paths() {
    let client = crate::all::create_client_from_struct!(Settings);
    let id = "document-empty-list-compat";
    client.put(settings(id, vec![])).run().await.unwrap();
    assert_eq!(
        client
            .get(id)
            .consistent()
            .run()
            .await
            .unwrap()
            .item
            .our_names
            .0,
        Vec::<String>::new()
    );

    for names in [vec![String::from("example")], vec![]] {
        client
            .update(id)
            .set(
                Settings::update_expression()
                    .set(Settings::our_names())
                    .value(Document::new(names.clone())),
            )
            .run()
            .await
            .unwrap();
        assert_eq!(
            client
                .get(id)
                .consistent()
                .run()
                .await
                .unwrap()
                .item
                .our_names
                .0,
            names
        );
    }

    client
        .batch_put(vec![settings(id, vec![])])
        .run()
        .await
        .unwrap();
    assert_eq!(
        client
            .get(id)
            .consistent()
            .run()
            .await
            .unwrap()
            .item
            .our_names
            .0,
        Vec::<String>::new()
    );

    #[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
    let tx = || {
        WriteTx::new(Region::Custom {
            endpoint: "http://127.0.0.1:8000".into(),
            name: "ap-northeast-1".into(),
        })
    };
    #[cfg(feature = "aws-sdk")]
    let tx = {
        let config = aws_sdk::aws_config::defaults(aws_sdk::config::BehaviorVersion::latest())
            .endpoint_url("http://127.0.0.1:8000")
            .region(config::Region::new("ap-northeast-1"))
            .load()
            .await;
        let sdk = Client::new(&config);
        move || WriteTx::new_with_client(sdk.clone())
    };
    tx().put(Settings::put(settings(id, vec![])))
        .run()
        .await
        .unwrap();
    assert_eq!(
        client
            .get(id)
            .consistent()
            .run()
            .await
            .unwrap()
            .item
            .our_names
            .0,
        Vec::<String>::new()
    );
    for names in [vec![String::from("example")], vec![]] {
        tx().update(
            Settings::update(id).set(
                Settings::update_expression()
                    .set(Settings::our_names())
                    .value(Document::new(names.clone())),
            ),
        )
        .run()
        .await
        .unwrap();
        assert_eq!(
            client
                .get(id)
                .consistent()
                .run()
                .await
                .unwrap()
                .item
                .our_names
                .0,
            names
        );
    }
    client.delete(id).run().await.unwrap();
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EmptySet {
    Strings,
    Numbers,
    Binary,
}

impl IntoAttribute for EmptySet {
    fn into_attr(self) -> AttributeValue {
        #[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
        return match self {
            Self::Strings => AttributeValue {
                ss: Some(vec![]),
                ..Default::default()
            },
            Self::Numbers => AttributeValue {
                ns: Some(vec![]),
                ..Default::default()
            },
            Self::Binary => AttributeValue {
                bs: Some(vec![]),
                ..Default::default()
            },
        };
        #[cfg(feature = "aws-sdk")]
        match self {
            Self::Strings => AttributeValue::Ss(vec![]),
            Self::Numbers => AttributeValue::Ns(vec![]),
            Self::Binary => AttributeValue::Bs(vec![]),
        }
    }
}

impl FromAttribute for EmptySet {
    fn from_attr(_: Option<AttributeValue>) -> Result<Self, ConversionError> {
        // Invalid empty sets must never be persisted or read back.
        Err(ConversionError::ValueIsNone)
    }
}

#[derive(Raiden, Debug, Clone)]
#[raiden(table_name = "user")]
#[allow(dead_code)] // Invalid empty sets must fail before any read can occur.
pub struct SourceIps {
    #[raiden(partition_key)]
    pub id: String,
    pub source_ips: EmptySet,
}

#[tokio::test]
async fn custom_empty_sets_are_rejected_instead_of_silently_omitted() {
    let client = crate::all::create_client_from_struct!(SourceIps);
    #[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
    let tx = || {
        WriteTx::new(Region::Custom {
            endpoint: "http://127.0.0.1:8000".into(),
            name: "ap-northeast-1".into(),
        })
    };
    #[cfg(feature = "aws-sdk")]
    let tx = {
        let config = aws_sdk::aws_config::defaults(aws_sdk::config::BehaviorVersion::latest())
            .endpoint_url("http://127.0.0.1:8000")
            .region(config::Region::new("ap-northeast-1"))
            .load()
            .await;
        let sdk = Client::new(&config);
        move || WriteTx::new_with_client(sdk.clone())
    };
    for kind in [EmptySet::Strings, EmptySet::Numbers, EmptySet::Binary] {
        let item = || SourceIpsPutItemInput {
            id: "invalid-empty-set-compat".into(),
            source_ips: kind,
        };
        let error = client.put(item()).run().await.unwrap_err();
        assert!(matches!(error, RaidenError::Validation(_)), "{error:?}");
        let error = client.batch_put(vec![item()]).run().await.unwrap_err();
        assert!(matches!(error, RaidenError::Validation(_)), "{error:?}");
        let error = tx().put(SourceIps::put(item())).run().await.unwrap_err();
        assert!(matches!(error, RaidenError::Validation(_)), "{error:?}");
        let expression = || {
            SourceIps::update_expression()
                .set(SourceIps::source_ips())
                .value(kind)
        };
        let error = client
            .update("invalid-empty-set-compat")
            .set(expression())
            .run()
            .await
            .unwrap_err();
        assert!(matches!(error, RaidenError::Validation(_)), "{error:?}");
        let error = tx()
            .update(SourceIps::update("invalid-empty-set-compat").set(expression()))
            .run()
            .await
            .unwrap_err();
        assert!(matches!(error, RaidenError::Validation(_)), "{error:?}");
    }
}
