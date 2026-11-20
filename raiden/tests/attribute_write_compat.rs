// Exercise the same DynamoDB JSON contract with each backend, without AWS access.
#![allow(clippy::result_large_err)]

use raiden::update_expression::{UpdateAddExpressionBuilder, UpdateDeleteExpressionBuilder};
use raiden::*;
use serde_json::{json, Value};
use std::collections::{BTreeSet, HashSet};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq)]
pub struct Raw(AttributeValue);

impl IntoAttribute for Raw {
    fn into_attr(self) -> AttributeValue {
        self.0
    }
}

impl FromAttribute for Raw {
    fn from_attr(value: Option<AttributeValue>) -> Result<Self, ConversionError> {
        value.map(Self).ok_or(ConversionError::ValueIsNone)
    }
}

#[derive(Raiden, Debug, Clone)]
pub struct AttributeWriteCompat {
    #[raiden(partition_key)]
    pub id: String,
    pub document: Document<Vec<String>>,
    pub raw: Raw,
    pub strings: HashSet<String>,
    pub ordered_strings: BTreeSet<String>,
    pub numbers: HashSet<usize>,
    pub ordered_numbers: BTreeSet<usize>,
    pub optional_set: Option<HashSet<String>>,
    pub vector: Vec<String>,
    pub nullable: Option<String>,
}

fn input(raw: AttributeValue) -> AttributeWriteCompatPutItemInput {
    AttributeWriteCompatPutItemInput {
        id: "compat".into(),
        document: Document::new(vec![]),
        raw: Raw(raw),
        strings: HashSet::new(),
        ordered_strings: BTreeSet::new(),
        numbers: HashSet::new(),
        ordered_numbers: BTreeSet::new(),
        optional_set: Some(HashSet::new()),
        vector: vec![],
        nullable: None,
    }
}

// One response per request, with bounded waits so a missing request cannot hang CI.
fn mock_response() -> (String, std::thread::JoinHandle<Value>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let endpoint = format!("http://{}", listener.local_addr().unwrap());
    let server = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(Instant::now() < deadline, "missing DynamoDB request");
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(e) => panic!("accept failed: {e}"),
            }
        };
        stream.set_nonblocking(false).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        let mut reader = BufReader::new(&mut stream);
        let mut length = None;
        loop {
            let mut line = String::new();
            assert_ne!(reader.read_line(&mut line).unwrap(), 0);
            if line == "\r\n" {
                break;
            }
            if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                length = Some(value.trim().parse::<usize>().unwrap());
            }
        }
        let mut body = vec![0; length.unwrap()];
        reader.read_exact(&mut body).unwrap();
        let request = serde_json::from_slice(&body).unwrap();
        write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/x-amz-json-1.0\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{{}}" ).unwrap();
        request
    });
    (endpoint, server)
}

#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
fn sdk_client(endpoint: String) -> (Client, Region) {
    (
        Client::new_with(
            StaticProvider::new_minimal("dummy".into(), "dummy".into()),
            HttpClient::new().unwrap(),
        ),
        Region::Custom {
            endpoint,
            name: "ap-northeast-1".into(),
        },
    )
}

#[cfg(feature = "aws-sdk")]
fn sdk_client(endpoint: String) -> Client {
    Client::from_conf(
        aws_sdk::config::Config::builder()
            .behavior_version(aws_sdk::config::BehaviorVersion::latest())
            .region(config::Region::new("ap-northeast-1"))
            .endpoint_url(endpoint)
            .credentials_provider(aws_credential_types::Credentials::new(
                "dummy", "dummy", None, None, "test",
            ))
            .build(),
    )
}

macro_rules! client {
    ($endpoint:expr) => {
        client!(AttributeWriteCompat, $endpoint)
    };
    ($model:ident, $endpoint:expr) => {{
        #[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
        let client = {
            let (client, region) = sdk_client($endpoint);
            $model::client_with(client, region)
        };
        #[cfg(feature = "aws-sdk")]
        let client = $model::client_with(sdk_client($endpoint));
        client
    }};
}

macro_rules! tx {
    ($endpoint:expr) => {{
        #[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
        let tx = {
            let (client, region) = sdk_client($endpoint);
            WriteTx::new_with_client(client, region)
        };
        #[cfg(feature = "aws-sdk")]
        let tx = WriteTx::new_with_client(sdk_client($endpoint));
        tx
    }};
}

macro_rules! attr {
    ($variant:ident, $field:ident, $value:expr) => {{
        #[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
        let value = AttributeValue {
            $field: Some($value),
            ..Default::default()
        };
        #[cfg(feature = "aws-sdk")]
        let value = AttributeValue::$variant($value);
        value
    }};
}

fn cases() -> Vec<(AttributeValue, Value)> {
    vec![
        (attr!(L, l, vec![]), json!({"L": []})),
        (attr!(Ss, ss, vec![]), json!({"SS": []})),
        (attr!(Ns, ns, vec![]), json!({"NS": []})),
        (attr!(Bs, bs, vec![]), json!({"BS": []})),
        (attr!(M, m, Default::default()), json!({"M": {}})),
        (attr!(S, s, String::new()), json!({"S": ""})),
        (attr!(B, b, Default::default()), json!({"B": ""})),
        (attr!(Null, null, true), json!({"NULL": true})),
        (attr!(Null, null, false), json!({"NULL": false})),
        (attr!(Bool, bool, false), json!({"BOOL": false})),
        (attr!(N, n, "0".into()), json!({"N": "0"})),
        (attr!(Ss, ss, vec!["a".into()]), json!({"SS": ["a"]})),
        (attr!(Ns, ns, vec!["1".into()]), json!({"NS": ["1"]})),
        (
            Document::new(json!([[], {}, null, ["x"]])).into_attr(),
            json!({"L": [{"L": []}, {"M": {}}, {"NULL": true}, {"L": [{"S": "x"}]}]}),
        ),
    ]
}

fn expected_item(raw: Value) -> Value {
    json!({
        "id": {"S": "compat"}, "document": {"L": []}, "raw": raw,
        "vector": {"NULL": true}, "nullable": {"NULL": true}
    })
}

#[tokio::test]
async fn put_batch_and_transaction_preserve_explicit_attributes() {
    for (raw, expected) in cases() {
        let expected = expected_item(expected);
        let (endpoint, server) = mock_response();
        client!(endpoint)
            .put(input(raw.clone()))
            .run()
            .await
            .unwrap();
        assert_eq!(server.join().unwrap()["Item"], expected);

        let (endpoint, server) = mock_response();
        client!(endpoint)
            .batch_put(vec![input(raw.clone())])
            .run()
            .await
            .unwrap();
        assert_eq!(
            server.join().unwrap()["RequestItems"]["AttributeWriteCompat"][0]["PutRequest"]["Item"],
            expected
        );

        let (endpoint, server) = mock_response();
        tx!(endpoint)
            .put(AttributeWriteCompat::put(input(raw)))
            .run()
            .await
            .unwrap();
        assert_eq!(
            server.join().unwrap()["TransactItems"][0]["Put"]["Item"],
            expected
        );
    }
}

fn assert_update(request: &Value, expected: Value) {
    let expression = request["UpdateExpression"].as_str().unwrap();
    assert!(expression.contains("SET"), "{request}");
    assert!(!expression.contains("REMOVE"), "{request}");
    let values = request["ExpressionAttributeValues"].as_object().unwrap();
    assert_eq!(values.len(), 1);
    assert_eq!(values.values().next().unwrap(), &expected);
}

#[tokio::test]
async fn updates_preserve_explicit_attributes() {
    for (raw, expected) in cases() {
        let (endpoint, server) = mock_response();
        client!(endpoint)
            .update("compat")
            .set(
                AttributeWriteCompat::update_expression()
                    .set(AttributeWriteCompat::raw())
                    .value(Raw(raw.clone())),
            )
            .run()
            .await
            .unwrap();
        assert_update(&server.join().unwrap(), expected.clone());

        let (endpoint, server) = mock_response();
        tx!(endpoint)
            .update(
                AttributeWriteCompat::update("compat").set(
                    AttributeWriteCompat::update_expression()
                        .set(AttributeWriteCompat::raw())
                        .value(Raw(raw)),
                ),
            )
            .run()
            .await
            .unwrap();
        assert_update(
            &server.join().unwrap()["TransactItems"][0]["Update"],
            expected,
        );
    }
}

#[tokio::test]
async fn empty_builtin_sets_still_remove_attributes_on_update() {
    let (endpoint, server) = mock_response();
    client!(endpoint)
        .update("compat")
        .set(
            AttributeWriteCompat::update_expression()
                .set(AttributeWriteCompat::strings())
                .value(HashSet::<String>::new()),
        )
        .set(
            AttributeWriteCompat::update_expression()
                .set(AttributeWriteCompat::ordered_strings())
                .value(BTreeSet::<String>::new()),
        )
        .set(
            AttributeWriteCompat::update_expression()
                .set(AttributeWriteCompat::numbers())
                .value(HashSet::<usize>::new()),
        )
        .set(
            AttributeWriteCompat::update_expression()
                .set(AttributeWriteCompat::ordered_numbers())
                .value(BTreeSet::<usize>::new()),
        )
        .set(
            AttributeWriteCompat::update_expression()
                .set(AttributeWriteCompat::optional_set())
                .value(Some(HashSet::<String>::new())),
        )
        .run()
        .await
        .unwrap();
    let request = server.join().unwrap();
    let expression = request["UpdateExpression"].as_str().unwrap();
    assert!(!expression.contains("SET"), "{request}");
    for name in [
        "strings",
        "ordered_strings",
        "numbers",
        "ordered_numbers",
        "optional_set",
    ] {
        assert!(expression.contains(&format!("#{name}")), "{request}");
    }
    assert!(expression.starts_with("REMOVE"), "{request}");
    assert!(
        request.get("ExpressionAttributeValues").is_none(),
        "{request}"
    );
}

#[test]
fn add_delete_preserve_explicit_empty_sets_and_skip_builtin_empty_sets() {
    for raw in [
        attr!(Ss, ss, vec![]),
        attr!(Ns, ns, vec![]),
        attr!(Bs, bs, vec![]),
    ] {
        let (expression, _, values) = AttributeWriteCompat::update_expression()
            .add(AttributeWriteCompat::raw())
            .value(Raw(raw.clone()))
            .build();
        assert!(!expression.is_empty());
        assert_eq!(values.values().collect::<Vec<_>>(), vec![&raw]);
        let (expression, _, values) = AttributeWriteCompat::update_expression()
            .delete(AttributeWriteCompat::raw())
            .value(Raw(raw.clone()))
            .build();
        assert!(!expression.is_empty());
        assert_eq!(values.values().collect::<Vec<_>>(), vec![&raw]);
    }
    let (expression, names, values) = AttributeWriteCompat::update_expression()
        .add(AttributeWriteCompat::strings())
        .value(HashSet::<String>::new())
        .build();
    assert!(expression.is_empty() && names.is_empty() && values.is_empty());
    let (expression, names, values) = AttributeWriteCompat::update_expression()
        .delete(AttributeWriteCompat::strings())
        .value(Some(HashSet::<String>::new()))
        .build();
    assert!(expression.is_empty() && names.is_empty() && values.is_empty());
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeliberatelyOmitted;

impl IntoAttribute for DeliberatelyOmitted {
    fn into_attr(self) -> AttributeValue {
        "unused fallback".into_attr()
    }

    fn into_attr_option(self) -> Option<AttributeValue> {
        None
    }
}

impl FromAttribute for DeliberatelyOmitted {
    fn from_attr(_: Option<AttributeValue>) -> Result<Self, ConversionError> {
        Ok(Self)
    }
}

#[derive(Raiden, Debug, Clone)]
pub struct OmissionCompat {
    #[raiden(partition_key)]
    pub id: String,
    pub omitted: Option<DeliberatelyOmitted>,
}

#[tokio::test]
async fn custom_omission_propagates_through_option_and_all_write_paths() {
    let input = || OmissionCompatPutItemInput {
        id: "compat".into(),
        omitted: Some(DeliberatelyOmitted),
    };
    let expected = json!({"id": {"S": "compat"}});
    let (endpoint, server) = mock_response();
    client!(OmissionCompat, endpoint)
        .put(input())
        .run()
        .await
        .unwrap();
    assert_eq!(server.join().unwrap()["Item"], expected);
    let (endpoint, server) = mock_response();
    client!(OmissionCompat, endpoint)
        .batch_put(vec![input()])
        .run()
        .await
        .unwrap();
    assert_eq!(
        server.join().unwrap()["RequestItems"]["OmissionCompat"][0]["PutRequest"]["Item"],
        expected
    );
    let (endpoint, server) = mock_response();
    tx!(endpoint)
        .put(OmissionCompat::put(input()))
        .run()
        .await
        .unwrap();
    assert_eq!(
        server.join().unwrap()["TransactItems"][0]["Put"]["Item"],
        expected
    );
    let set = || {
        OmissionCompat::update_expression()
            .set(OmissionCompat::omitted())
            .value(Some(DeliberatelyOmitted))
    };
    let (endpoint, server) = mock_response();
    client!(OmissionCompat, endpoint)
        .update("compat")
        .set(set())
        .run()
        .await
        .unwrap();
    let request = server.join().unwrap();
    assert_eq!(
        request["UpdateExpression"].as_str().unwrap().trim(),
        "REMOVE #omitted"
    );
    assert!(request.get("ExpressionAttributeValues").is_none());
    let (endpoint, server) = mock_response();
    tx!(endpoint)
        .update(OmissionCompat::update("compat").set(set()))
        .run()
        .await
        .unwrap();
    let request = server.join().unwrap();
    assert_eq!(
        request["TransactItems"][0]["Update"]["UpdateExpression"]
            .as_str()
            .unwrap()
            .trim(),
        "REMOVE #omitted"
    );
}

#[test]
fn nonempty_builtin_sets_are_preserved() {
    assert_eq!(
        HashSet::from(["a".to_owned()]).into_attr_option(),
        Some(attr!(Ss, ss, vec!["a".into()]))
    );
    assert_eq!(
        BTreeSet::from(["a".to_owned()]).into_attr_option(),
        Some(attr!(Ss, ss, vec!["a".into()]))
    );
    macro_rules! numeric_sets {
        ($($ty:ty),*) => {$(
            assert_eq!(HashSet::from([1 as $ty]).into_attr_option(), Some(attr!(Ns, ns, vec!["1".into()])));
            assert_eq!(BTreeSet::from([1 as $ty]).into_attr_option(), Some(attr!(Ns, ns, vec!["1".into()])));
            assert_eq!(HashSet::<$ty>::new().into_attr_option(), None);
            assert_eq!(BTreeSet::<$ty>::new().into_attr_option(), None);
        )*};
    }
    numeric_sets!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);
}
