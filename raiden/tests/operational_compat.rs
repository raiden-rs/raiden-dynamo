//! Identical scripted DynamoDB responses on both backends, without AWS access.
//! SDK retries are disabled: retry tests exercise Raiden's explicit policy only.
#![allow(clippy::result_large_err)]
use raiden::*;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::time::{Duration, Instant};

#[derive(Raiden, Debug, PartialEq, serde::Serialize)]
pub struct OperationalCompat {
    #[raiden(partition_key)]
    pub id: String,
}

#[derive(Raiden, Debug, PartialEq, serde::Serialize)]
#[raiden(gsi(name = "note-index", partition_key = "note"))]
pub struct OperationalCompatWrite {
    #[raiden(partition_key)]
    pub id: String,
    pub note: Option<String>,
}

#[derive(RaidenIndex, Debug, PartialEq)]
#[raiden(source = "OperationalCompatWrite", gsi = "note-index")]
pub struct OperationalCompatNote {
    pub note: Option<String>,
}

// Script exactly N responses. A surprise request fails promptly, including
// accidental pagination after an empty key. Every expected request has a deadline.
fn serve(responses: Vec<(u16, Value)>) -> (String, std::thread::JoinHandle<Vec<Value>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let endpoint = format!("http://{}", listener.local_addr().unwrap());
    let server = std::thread::spawn(move || {
        let mut requests = vec![];
        for (status, response) in responses {
            let deadline = Instant::now() + Duration::from_secs(10);
            let mut stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(Instant::now() < deadline, "missing DynamoDB request");
                        std::thread::sleep(Duration::from_millis(5));
                    }
                    Err(e) => panic!("accept: {e}"),
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
            requests.push(serde_json::from_slice(&body).unwrap());
            let response = response.to_string();
            write!(stream, "HTTP/1.1 {status} Test\r\nContent-Type: application/x-amz-json-1.0\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response}", response.len()).unwrap();
        }
        requests
    });
    (endpoint, server)
}

#[cfg(feature = "rusoto")]
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
            .retry_config(aws_sdk::config::retry::RetryConfig::disabled())
            .credentials_provider(aws_credential_types::Credentials::new(
                "dummy", "dummy", None, None, "test",
            ))
            .build(),
    )
}

macro_rules! client {
    ($endpoint:expr) => {
        client!(OperationalCompat, $endpoint)
    };
    ($model:ident, $endpoint:expr) => {{
        #[cfg(feature = "rusoto")]
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
        #[cfg(feature = "rusoto")]
        let tx = {
            let (client, region) = sdk_client($endpoint);
            WriteTx::new_with_client(client, region)
        };
        #[cfg(feature = "aws-sdk")]
        let tx = WriteTx::new_with_client(sdk_client($endpoint));
        tx
    }};
}

fn page(items: Vec<Value>, scanned: u32, key: Option<Value>) -> Value {
    let mut result = json!({"Count": items.len(), "ScannedCount": scanned, "Items": items});
    if let Some(key) = key {
        result["LastEvaluatedKey"] = key;
    }
    result
}
fn key(id: &str) -> Value {
    json!({"id": {"S": id}})
}

const ERROR_OPERATIONS: [&str; 11] = [
    "get",
    "batch_get",
    "transact_get",
    "query",
    "scan",
    "put",
    "batch_put",
    "batch_delete",
    "delete",
    "update",
    "transact_write",
];

#[tokio::test]
async fn service_error_messages_and_new_sdk_errors_keep_the_rusoto_contract() {
    for operation in ERROR_OPERATIONS {
        for message in [Some("detail あ\n😀"), Some(""), None] {
            for code in [
                "InternalServerError",
                "ValidationException",
                "InvalidEndpointException",
                "ThrottlingException",
            ] {
                let mut body = json!({"__type": code});
                if let Some(message) = message {
                    body["message"] = json!(message);
                }
                let observed = observe_error(operation, 400, body).await;
                let (kind, retryable, expected_message) = match code {
                    "InternalServerError" => (
                        "InternalServerError",
                        true,
                        json!(message.unwrap_or_default()),
                    ),
                    "ValidationException" => {
                        ("Validation", false, json!(message.unwrap_or_default()))
                    }
                    _ => ("Unknown", true, Value::Null),
                };
                assert_eq!(
                    observed["error"],
                    json!({"kind": kind, "message": expected_message, "retryable": retryable}),
                    "{operation}/{code}/{message:?}"
                );
                let requests = observed["requests"].as_array().unwrap();
                assert_eq!(requests.len(), 1);
                #[cfg(feature = "rusoto")]
                assert!(requests[0].get("ClientRequestToken").is_none());
                #[cfg(feature = "aws-sdk")]
                assert_eq!(
                    requests[0].get("ClientRequestToken").is_some(),
                    operation == "transact_write"
                );
            }
        }
    }
}

#[tokio::test]
async fn error_body_decoding_preserves_legacy_precedence_and_validation() {
    for operation in ERROR_OPERATIONS {
        for (body, kind, message, retryable) in [
            (
                json!({"code": "ValidationException", "message": "detail"}),
                "Unknown",
                Value::Null,
                true,
            ),
            (
                json!({"Code": "InternalServerError"}),
                "Unknown",
                Value::Null,
                true,
            ),
            (
                json!({"__type": "InternalServerError:detail"}),
                "Unknown",
                Value::Null,
                true,
            ),
            (
                json!({"__type": "prefix#nested#InternalServerError", "Message": "detail"}),
                "InternalServerError",
                json!("detail"),
                true,
            ),
            (
                json!({"__type": "InternalServerError", "code": "ValidationException", "message": "detail"}),
                "InternalServerError",
                json!("detail"),
                true,
            ),
            (
                json!({"__type": "ValidationException", "message": 1}),
                "Unknown",
                Value::Null,
                true,
            ),
            (
                json!({"__type": "ValidationException", "message": "lower", "Message": "upper"}),
                "Unknown",
                Value::Null,
                true,
            ),
        ] {
            let observed = observe_error(operation, 400, body.clone()).await;
            assert_eq!(
                observed["error"],
                json!({"kind": kind, "message": message, "retryable": retryable}),
                "{operation}/{body}"
            );
        }
    }
    for operation in ["transact_get", "transact_write"] {
        let body = json!({"__type": "TransactionCanceledException",
            "message": "Transaction cancelled, please refer cancellation reasons for specific reasons [ConditionalCheckFailed]",
            "CancellationReasons": "invalid"});
        let observed = observe_error(operation, 400, body).await;
        assert_eq!(observed["error"]["kind"], "TransactionCanceled");
        assert_eq!(observed["error"]["retryable"], false);
        assert_eq!(
            observed["error"]["reasons"],
            "RaidenTransactionCancellationReasons([Some(ConditionalCheckFailed)])"
        );
    }
}

// Preserve the variant and application-visible message, not backend-specific
// transport/debug types. Also observe the shared retry policy's decision.
fn error_observation(error: &RaidenError) -> Value {
    use RaidenError::*;
    let (kind, message) = match error {
        ConditionalCheckFailed(s) => ("ConditionalCheckFailed", Some(s)),
        IdempotentParameterMismatch(s) => ("IdempotentParameterMismatch", Some(s)),
        InternalServerError(s) => ("InternalServerError", Some(s)),
        ItemCollectionSizeLimitExceeded(s) => ("ItemCollectionSizeLimitExceeded", Some(s)),
        ProvisionedThroughputExceeded(s) => ("ProvisionedThroughputExceeded", Some(s)),
        RequestLimitExceeded(s) => ("RequestLimitExceeded", Some(s)),
        ResourceNotFound(s) => ("ResourceNotFound", Some(s)),
        SizeLimitExceeded(s) => ("SizeLimitExceeded", Some(s)),
        TransactionConflict(s) => ("TransactionConflict", Some(s)),
        TransactionInProgress(s) => ("TransactionInProgress", Some(s)),
        Validation(s) => ("Validation", Some(s)),
        Unknown(_) => ("Unknown", None),
        TransactionCanceled { reasons, .. } => {
            return json!({"kind": "TransactionCanceled", "reasons": format!("{reasons:?}"),
                "retryable": DefaultRetryStrategy.should_retry(error)});
        }
        _ => return json!({"other": format!("{error:?}")}),
    };
    json!({"kind": kind, "message": message,
        "retryable": DefaultRetryStrategy.should_retry(error)})
}

async fn observe_error(operation: &str, status: u16, response: Value) -> Value {
    let (endpoint, server) = serve(vec![(status, response)]);
    let client = client!(endpoint.clone()).with_retries(Box::new(NopRetryStrategy));
    let input = || OperationalCompatPutItemInput { id: "id".into() };
    let error = match operation {
        "get" => client.get("id").run().await.unwrap_err(),
        "batch_get" => client.batch_get(vec!["id"]).run().await.unwrap_err(),
        "transact_get" => client.transact_get(vec!["id"]).run().await.unwrap_err(),
        "query" => client
            .query()
            .key_condition(OperationalCompat::key_condition(OperationalCompat::id()).eq("id"))
            .run()
            .await
            .unwrap_err(),
        "scan" => client.scan().run().await.unwrap_err(),
        "put" => client.put(input()).run().await.unwrap_err(),
        "batch_put" => client.batch_put(vec![input()]).run().await.unwrap_err(),
        "batch_delete" => client.batch_delete(vec!["id"]).run().await.unwrap_err(),
        "delete" => client.delete("id").run().await.unwrap_err(),
        // Error-response decoding only: this fixture does not assert DynamoDB
        // would accept a primary-key update.
        "update" => client
            .update("id")
            .set(
                OperationalCompat::update_expression()
                    .set(OperationalCompat::id())
                    .value("next"),
            )
            .run()
            .await
            .unwrap_err(),
        "transact_write" => tx!(endpoint)
            .with_retries(Box::new(NopRetryStrategy))
            .put(OperationalCompat::put(input()))
            .run()
            .await
            .unwrap_err(),
        _ => panic!("unknown operation: {operation}"),
    };
    json!({"error": error_observation(&error), "requests": server.join().unwrap()})
}

/// Driven by scripts/check-operational-backends.py, in two feature builds.
#[tokio::test]
#[ignore = "requires the differential corpus/output environment variables"]
async fn differential_service_error_corpus() {
    let corpus = std::fs::read_to_string(std::env::var("RAIDEN_ERROR_CORPUS").unwrap()).unwrap();
    let mut output = std::fs::File::create(std::env::var("RAIDEN_ERROR_OUTPUT").unwrap()).unwrap();
    for line in corpus.lines() {
        let case: Value = serde_json::from_str(line).unwrap();
        let observed = observe_error(
            case["operation"].as_str().unwrap(),
            case["status"].as_u64().unwrap().try_into().unwrap(),
            case["body"].clone(),
        )
        .await;
        writeln!(output, "{observed}").unwrap();
    }
}

fn result_observation<T: serde::Serialize>(result: Result<T, RaidenError>) -> Value {
    match result {
        Ok(output) => json!({"ok": output}),
        Err(error) => json!({"error": error_observation(&error)}),
    }
}

#[tokio::test]
#[ignore = "requires the differential corpus/output environment variables"]
async fn differential_success_response_corpus() {
    let corpus = std::fs::read_to_string(std::env::var("RAIDEN_ERROR_CORPUS").unwrap()).unwrap();
    let mut output = std::fs::File::create(std::env::var("RAIDEN_ERROR_OUTPUT").unwrap()).unwrap();
    for line in corpus.lines() {
        let case: Value = serde_json::from_str(line).unwrap();
        let (endpoint, server) = serve(vec![(200, case["body"].clone())]);
        let client = client!(endpoint.clone()).with_retries(Box::new(NopRetryStrategy));
        let input = || OperationalCompatPutItemInput { id: "id".into() };
        let observed = match case["operation"].as_str().unwrap() {
            "get" => result_observation(client.get("id").run().await),
            "batch_get" => result_observation(client.batch_get(vec!["id"]).run().await),
            "transact_get" => result_observation(client.transact_get(vec!["id"]).run().await),
            "query" => result_observation(
                client
                    .query()
                    .key_condition(
                        OperationalCompat::key_condition(OperationalCompat::id()).eq("id"),
                    )
                    .run()
                    .await,
            ),
            "scan" => result_observation(client.scan().run().await),
            "put" => result_observation(client.put(input()).run().await.map(|output| {
                raiden::put::PutOutput {
                    consumed_capacity: output.consumed_capacity,
                    item: OperationalCompat { id: output.item.id },
                }
            })),
            "delete" => result_observation(client.delete("id").run().await),
            "batch_put" => result_observation(client.batch_put(vec![input()]).run().await),
            "batch_delete" => result_observation(client.batch_delete(vec!["id"]).run().await),
            "transact_write" => result_observation(
                tx!(endpoint)
                    .with_retries(Box::new(NopRetryStrategy))
                    .put(OperationalCompat::put(input()))
                    .run()
                    .await,
            ),
            "update" => {
                let client = client!(OperationalCompatWrite, endpoint)
                    .with_retries(Box::new(NopRetryStrategy));
                let builder = client.update("id").set(
                    OperationalCompatWrite::update_expression()
                        .set(OperationalCompatWrite::note())
                        .value(Some("new".to_owned())),
                );
                let builder = match case["return_values"].as_str() {
                    Some("ALL_OLD") => builder.return_all_old(),
                    Some("ALL_NEW") => builder.return_all_new(),
                    None => builder,
                    Some(value) => panic!("unknown ReturnValues: {value}"),
                };
                result_observation(builder.run().await)
            }
            operation => panic!("unknown operation: {operation}"),
        };
        writeln!(
            output,
            "{}",
            json!({"result": observed, "requests": server.join().unwrap()})
        )
        .unwrap();
    }
}

#[tokio::test]
async fn query_and_scan_continue_after_empty_page_and_reduce_evaluated_limit() {
    for scan in [false, true] {
        let (endpoint, server) = serve(vec![
            (200, page(vec![], 2, Some(key("first")))),
            (200, page(vec![key("second")], 1, None)),
        ]);
        let client = client!(endpoint);
        if scan {
            let result = client.scan().limit(3).run().await.unwrap();
            assert_eq!(
                result.items,
                vec![OperationalCompat {
                    id: "second".into()
                }]
            );
            assert!(result.last_evaluated_key.is_none());
        } else {
            let condition = OperationalCompat::key_condition(OperationalCompat::id()).eq("first");
            let result = client
                .query()
                .key_condition(condition)
                .limit(3)
                .run()
                .await
                .unwrap();
            assert_eq!(
                result.items,
                vec![OperationalCompat {
                    id: "second".into()
                }]
            );
            assert!(result.next_token.is_none());
        }
        let requests = server.join().unwrap();
        assert_eq!(requests[0]["Limit"], 3);
        assert!(requests[0].get("ExclusiveStartKey").is_none());
        assert_eq!(requests[1]["Limit"], 1);
        assert_eq!(requests[1]["ExclusiveStartKey"], key("first"));
    }
}

#[tokio::test]
async fn query_limit_returns_a_resumable_token_even_when_all_items_are_filtered_out() {
    let (endpoint, server) = serve(vec![(200, page(vec![], 2, Some(key("cursor"))))]);
    let client = client!(endpoint);
    let condition = OperationalCompat::key_condition(OperationalCompat::id()).eq("first");
    let result = client
        .query()
        .key_condition(condition)
        .limit(2)
        .run()
        .await
        .unwrap();
    assert!(result.items.is_empty());
    let token = result.next_token.unwrap();
    server.join().unwrap();
    let (endpoint, server) = serve(vec![(200, page(vec![key("after")], 1, None))]);
    let client = client!(endpoint);
    let condition = OperationalCompat::key_condition(OperationalCompat::id()).eq("first");
    let result = client
        .query()
        .key_condition(condition)
        .next_token(token)
        .limit(2)
        .run()
        .await
        .unwrap();
    assert_eq!(result.items[0].id, "after");
    assert!(result.next_token.is_none());
    assert_eq!(
        server.join().unwrap()[0]["ExclusiveStartKey"],
        key("cursor")
    );
}

#[tokio::test]
async fn query_and_scan_stop_on_explicit_empty_last_evaluated_key() {
    for scan in [false, true] {
        let (endpoint, server) = serve(vec![(200, page(vec![key("last")], 1, Some(json!({}))))]);
        let client = client!(endpoint);
        if scan {
            let result = client.scan().run().await.unwrap();
            assert_eq!(result.items.len(), 1);
            assert!(result.last_evaluated_key.is_none());
        } else {
            let condition = OperationalCompat::key_condition(OperationalCompat::id()).eq("last");
            let result = client.query().key_condition(condition).run().await.unwrap();
            assert_eq!(result.items.len(), 1);
            assert!(result.next_token.is_none());
        }
        assert_eq!(server.join().unwrap().len(), 1);
    }
}

#[derive(Clone, Copy)]
struct RetryTwice;
impl RetryStrategy for RetryTwice {
    fn should_retry(&self, error: &RaidenError) -> bool {
        DefaultRetryStrategy.should_retry(error)
    }
    fn policy(&self) -> Policy {
        Policy::Pause(2, Duration::ZERO)
    }
}
fn service_error(code: &str) -> Value {
    json!({"__type": format!("com.amazonaws.dynamodb.v20120810#{code}"), "message": "scripted failure"})
}

#[tokio::test]
async fn explicit_retry_policy_retries_transient_errors_and_preserves_the_final_error() {
    for code in [
        "InternalServerError",
        "ProvisionedThroughputExceededException",
        "RequestLimitExceeded",
    ] {
        for succeeds in [false, true] {
            let status = if code == "InternalServerError" {
                500
            } else {
                400
            };
            let last = if succeeds {
                (200, json!({"Item": key("found")}))
            } else {
                (status, service_error(code))
            };
            let (endpoint, server) = serve(vec![
                (status, service_error(code)),
                (status, service_error(code)),
                last,
            ]);
            let client = client!(endpoint).with_retries(Box::new(RetryTwice));
            let result = client.get("found").run().await;
            if succeeds {
                assert_eq!(result.unwrap().item.id, "found");
            } else {
                let error = result.unwrap_err();
                assert!(matches!(
                    (code, error),
                    ("InternalServerError", RaidenError::InternalServerError(_))
                        | (
                            "ProvisionedThroughputExceededException",
                            RaidenError::ProvisionedThroughputExceeded(_)
                        )
                        | ("RequestLimitExceeded", RaidenError::RequestLimitExceeded(_))
                ));
            }
            let requests = server.join().unwrap();
            assert_eq!(requests.len(), 3);
            assert!(requests.windows(2).all(|pair| pair[0] == pair[1]));
        }
    }
}

#[tokio::test]
async fn typed_index_scan_uses_the_explicit_retry_policy() {
    let item = json!({"note": {"S": "found"}});
    let (endpoint, server) = serve(vec![
        (500, service_error("InternalServerError")),
        (200, page(vec![item], 1, None)),
    ]);
    let client = client!(OperationalCompatWrite, endpoint).with_retries(Box::new(RetryTwice));
    let result = OperationalCompatNote::scan(&client).run().await.unwrap();
    assert_eq!(
        result.items,
        vec![OperationalCompatNote {
            note: Some("found".into()),
        }]
    );

    let requests = server.join().unwrap();
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0], requests[1]);
    assert_eq!(requests[0]["IndexName"], "note-index");
}

#[tokio::test]
async fn explicit_retry_policy_applies_to_scan_delete_and_batch_requests() {
    let put = |id: &str| OperationalCompatPutItemInput { id: id.into() };
    let put_request = json!({"PutRequest": {"Item": key("pending")}});
    let delete_request = json!({"DeleteRequest": {"Key": key("pending")}});
    let pending_get = json!({"Keys": [key("pending")]});
    for (operation, first, follow_up, pending) in [
        (
            "scan",
            json!({"Items": [key("found")], "Count": 1, "ScannedCount": 1,
                   "LastEvaluatedKey": key("found")}),
            Some(json!({"Items": [key("next")], "Count": 1, "ScannedCount": 1})),
            Some(key("found")),
        ),
        ("delete", json!({}), None, None),
        (
            "batch_get",
            json!({"Responses": {"OperationalCompat": [key("found")]},
                   "UnprocessedKeys": {"OperationalCompat": pending_get}}),
            Some(json!({"Responses": {"OperationalCompat": [key("pending")]}})),
            Some(json!([key("pending")])),
        ),
        (
            "batch_put",
            json!({"UnprocessedItems": {"OperationalCompat": [put_request]}}),
            Some(json!({})),
            Some(json!([put_request])),
        ),
        (
            "batch_delete",
            json!({"UnprocessedItems": {"OperationalCompat": [delete_request]}}),
            Some(json!({})),
            Some(json!([delete_request])),
        ),
    ] {
        let transient = (500, service_error("InternalServerError"));
        let mut responses = vec![transient.clone(), (200, first)];
        if let Some(follow_up) = follow_up {
            responses.extend([transient, (200, follow_up)]);
        }
        let (endpoint, server) = serve(responses);
        let client = client!(endpoint).with_retries(Box::new(RetryTwice));
        let result = match operation {
            "scan" => client.scan().run().await.map(|output| {
                let ids: Vec<_> = output.items.into_iter().map(|item| item.id).collect();
                assert_eq!(ids, ["found", "next"]);
            }),
            "delete" => client.delete("found").run().await,
            "batch_get" => client
                .batch_get(vec!["found", "pending"])
                .run()
                .await
                .map(|output| {
                    let ids: Vec<_> = output.items.into_iter().map(|item| item.id).collect();
                    assert_eq!(ids, ["found", "pending"]);
                }),
            "batch_put" => client
                .batch_put(vec![put("found"), put("pending")])
                .run()
                .await
                .map(|output| {
                    assert!(output.unprocessed_items.is_empty());
                }),
            "batch_delete" => client
                .batch_delete(vec!["found", "pending"])
                .run()
                .await
                .map(|output| {
                    assert!(output.unprocessed_items.is_empty());
                }),
            _ => unreachable!(),
        };
        result.unwrap_or_else(|error| panic!("{operation}: {error:?}"));
        let requests = server.join().unwrap();
        assert_eq!(requests[0], requests[1], "{operation}");
        if let Some(pending) = pending {
            assert_eq!(requests[2], requests[3], "{operation}");
            let request_items = &requests[2]["RequestItems"]["OperationalCompat"];
            let actual = match operation {
                "scan" => &requests[2]["ExclusiveStartKey"],
                "batch_get" => &request_items["Keys"],
                _ => request_items,
            };
            assert_eq!(actual, &pending, "{operation}");
        }
    }
}

#[tokio::test]
async fn condition_failure_is_not_retried_even_with_an_explicit_retry_policy() {
    let (endpoint, server) = serve(vec![(
        400,
        service_error("ConditionalCheckFailedException"),
    )]);
    let client = client!(endpoint).with_retries(Box::new(RetryTwice));
    let result = client
        .put(OperationalCompatPutItemInput { id: "same".into() })
        .run()
        .await;
    assert!(matches!(
        result,
        Err(RaidenError::ConditionalCheckFailed(_))
    ));
    assert_eq!(server.join().unwrap().len(), 1);
}

#[tokio::test]
async fn transaction_retries_preserve_payload_and_backend_token_contract() {
    let (endpoint, server) = serve(vec![
        (500, service_error("InternalServerError")),
        (500, service_error("InternalServerError")),
        (200, json!({})),
    ]);
    tx!(endpoint)
        .with_retries(Box::new(RetryTwice))
        .put(OperationalCompat::put(OperationalCompatPutItemInput {
            id: "a".into(),
        }))
        .put(OperationalCompat::put(OperationalCompatPutItemInput {
            id: "b".into(),
        }))
        .run()
        .await
        .unwrap();
    let requests = server.join().unwrap();
    assert_eq!(requests.len(), 3);
    #[cfg(feature = "aws-sdk")]
    let mut requests = requests;
    #[cfg(feature = "aws-sdk")]
    {
        let token = requests[0]["ClientRequestToken"]
            .as_str()
            .unwrap()
            .to_owned();
        uuid::Uuid::parse_str(&token).unwrap();
        assert!(requests
            .iter()
            .all(|request| request["ClientRequestToken"] == token));
        for request in &mut requests {
            request
                .as_object_mut()
                .unwrap()
                .remove("ClientRequestToken");
        }
        // A separate logical transaction must receive a fresh token.
        let (endpoint, server) = serve(vec![(200, json!({}))]);
        tx!(endpoint)
            .put(OperationalCompat::put(OperationalCompatPutItemInput {
                id: "a".into(),
            }))
            .run()
            .await
            .unwrap();
        assert_ne!(server.join().unwrap()[0]["ClientRequestToken"], token);
    }
    let expected = json!({"TransactItems": [
        {"Put": {"Item": {"id": {"S": "a"}}, "TableName": "OperationalCompat"}},
        {"Put": {"Item": {"id": {"S": "b"}}, "TableName": "OperationalCompat"}},
    ]});
    assert!(requests.iter().all(|request| request == &expected));
}

#[tokio::test]
#[cfg(feature = "aws-sdk")]
async fn transaction_token_is_stable_across_nested_sdk_and_raiden_retries() {
    let mut responses = vec![(500, service_error("InternalServerError")); 5];
    responses.push((200, json!({})));
    let (endpoint, server) = serve(responses);
    let config = sdk_client(endpoint)
        .config()
        .to_builder()
        .retry_config(
            aws_sdk::config::retry::RetryConfig::standard()
                .with_max_attempts(2)
                .with_initial_backoff(Duration::from_millis(1)),
        )
        .build();
    WriteTx::new_with_client(Client::from_conf(config))
        .with_retries(Box::new(RetryTwice))
        .put(OperationalCompat::put(OperationalCompatPutItemInput {
            id: "a".into(),
        }))
        .run()
        .await
        .unwrap();
    let requests = server.join().unwrap();
    assert_eq!(requests.len(), 6);
    let token = requests[0]["ClientRequestToken"].as_str().unwrap();
    uuid::Uuid::parse_str(token).unwrap();
    assert!(requests.iter().all(|request| request == &requests[0]));
}

#[tokio::test]
async fn transaction_cancellation_retains_reason_order_for_backend_business_rules() {
    let codes = [
        "ConditionalCheckFailed",
        "None",
        "TransactionConflict",
        "ThrottlingError",
        "ValidationError",
        "ItemCollectionSizeLimitExceeded",
        "ProvisionedThroughputExceeded",
        "FutureReasonCode",
    ];
    let message = format!(
        "Transaction cancelled, please refer cancellation reasons for specific reasons [{}]",
        codes.join(", ")
    );
    for read in [false, true] {
        let response = json!({"__type": "com.amazonaws.dynamodb.v20120810#TransactionCanceledException", "Message": message,
            "CancellationReasons": codes.iter().map(|code| json!({"Code": code, "Message": "detail"})).collect::<Vec<_>>()});
        let (endpoint, server) = serve(vec![(400, response)]);
        let ids = (0..codes.len())
            .map(|i| format!("item-{i}"))
            .collect::<Vec<_>>();
        let error = if read {
            client!(endpoint).transact_get(ids).run().await.unwrap_err()
        } else {
            let mut transaction = tx!(endpoint);
            for id in ids {
                transaction =
                    transaction.put(OperationalCompat::put(OperationalCompatPutItemInput { id }));
            }
            transaction.run().await.unwrap_err()
        };
        let RaidenError::TransactionCanceled { reasons, .. } = error else {
            panic!("unexpected error: {error:?}")
        };
        assert_eq!(
            reasons.0,
            vec![
                Some(RaidenTransactionCancellationReason::ConditionalCheckFailed),
                None,
                Some(RaidenTransactionCancellationReason::TransactionConflict),
                Some(RaidenTransactionCancellationReason::ThrottlingError),
                Some(RaidenTransactionCancellationReason::ValidationError),
                Some(RaidenTransactionCancellationReason::ItemCollectionSizeLimitExceeded),
                Some(RaidenTransactionCancellationReason::ProvisionedThroughputExceeded),
                Some(RaidenTransactionCancellationReason::Unknown),
            ]
        );
        assert!(reasons.has_conditional_check_failed());
        assert!(reasons.has_transaction_conflict());
        assert!(reasons.has_throttling_error());
        assert!(reasons.has_validation_error());
        assert!(reasons.has_item_collection_size_limit_exceeded());
        assert!(reasons.has_provisioned_throughput_exceeded());
        let requests = server.join().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0]["TransactItems"].as_array().unwrap().len(),
            codes.len()
        );
    }
}

fn cursor_json(attributes: AttributeValues) -> Value {
    key(&String::from_attr(attributes.get("id").cloned()).unwrap())
}

// Enumerate page shapes and budgets independently of the pagination loop.
// Includes empty intermediate pages, both terminal-key encodings, and stopping
// at a limit before the terminal page. Each captured request must resume at the
// previous page's cursor and use the remaining evaluated-item budget.
#[tokio::test]
async fn pagination_page_shapes_and_limit_boundaries_match_the_model() {
    for scan in [false, true] {
        for shape in 0..8 {
            for budget in [None, Some(1), Some(2), Some(3), Some(5)] {
                let evaluated = [1, 2, 1];
                let mut remaining = budget;
                let mut expected_items = vec![];
                let mut expected_requests = vec![];
                let mut responses = vec![];
                let mut expected_cursor = None;
                for (page_index, scanned) in evaluated.into_iter().enumerate() {
                    let scanned = remaining.map_or(scanned, |limit| scanned.min(limit));
                    let items = if shape & (1 << page_index) == 0 {
                        vec![]
                    } else {
                        vec![key(&format!("page-{page_index}"))]
                    };
                    expected_items.extend(items.iter().map(|item| OperationalCompat {
                        id: item["id"]["S"].as_str().unwrap().to_owned(),
                    }));
                    expected_requests.push((remaining, expected_cursor.clone()));
                    // Alternate omitted and explicit empty terminal keys.
                    expected_cursor =
                        (page_index < 2).then(|| key(&format!("cursor-{page_index}")));
                    let wire_cursor = expected_cursor
                        .clone()
                        .or_else(|| (shape % 2 == 0).then(|| json!({})));
                    responses.push((200, page(items, scanned, wire_cursor)));
                    if let Some(limit) = remaining {
                        remaining = Some(limit - scanned);
                        if remaining == Some(0) {
                            break;
                        }
                    }
                }
                let (endpoint, server) = serve(responses);
                let client = client!(endpoint);
                let cursor = if scan {
                    let mut builder = client.scan();
                    if let Some(budget) = budget {
                        builder = builder.limit(budget as usize);
                    }
                    let result = builder.run().await.unwrap();
                    assert_eq!(
                        result.items, expected_items,
                        "scan shape={shape} budget={budget:?}"
                    );
                    result.last_evaluated_key.map(cursor_json)
                } else {
                    let condition =
                        OperationalCompat::key_condition(OperationalCompat::id()).eq("partition");
                    let mut builder = client.query().key_condition(condition);
                    if let Some(budget) = budget {
                        builder = builder.limit(budget as usize);
                    }
                    let result = builder.run().await.unwrap();
                    assert_eq!(
                        result.items, expected_items,
                        "query shape={shape} budget={budget:?}"
                    );
                    result
                        .next_token
                        .map(|token| cursor_json(token.into_attr_values().unwrap()))
                };
                assert_eq!(
                    cursor, expected_cursor,
                    "scan={scan} shape={shape} budget={budget:?}"
                );
                let requests = server.join().unwrap();
                assert_eq!(requests.len(), expected_requests.len());
                for (request, (limit, cursor)) in requests.iter().zip(expected_requests) {
                    assert_eq!(
                        request.get("Limit"),
                        limit.map(|limit| json!(limit)).as_ref()
                    );
                    assert_eq!(request.get("ExclusiveStartKey"), cursor.as_ref());
                }
            }
        }
    }
}

#[tokio::test]
async fn optional_page_counters_preserve_presence_and_final_page_values() {
    for scan in [false, true] {
        for count in [None, Some(Value::Null), Some(json!(0)), Some(json!(2))] {
            for scanned in [None, Some(Value::Null), Some(json!(0)), Some(json!(2))] {
                let mut last = json!({"Items": []});
                if let Some(value) = &count {
                    last["Count"] = value.clone();
                }
                if let Some(value) = &scanned {
                    last["ScannedCount"] = value.clone();
                }
                // First-page values must not leak into an absent final counter.
                let first = json!({"Items": [], "Count": 7, "ScannedCount": 8,
                    "LastEvaluatedKey": {"id": {"S": "cursor"}}});
                let (endpoint, server) = serve(vec![(200, first), (200, last)]);
                let client = client!(endpoint);
                let observed = if scan {
                    let result = client.scan().run().await.unwrap();
                    (result.count, result.scanned_count)
                } else {
                    let condition =
                        OperationalCompat::key_condition(OperationalCompat::id()).eq("id");
                    let result = client.query().key_condition(condition).run().await.unwrap();
                    (result.count, result.scanned_count)
                };
                assert_eq!(
                    observed,
                    (
                        count.as_ref().and_then(|v| v.as_i64()),
                        scanned.and_then(|v| v.as_i64())
                    )
                );
                assert_eq!(server.join().unwrap().len(), 2);
            }
        }
    }
}

#[cfg(feature = "aws-sdk")]
#[tokio::test]
async fn page_counter_capture_retains_user_interceptors() {
    use aws_smithy_runtime_api::{
        box_error::BoxError,
        client::{
            interceptors::{context::AfterDeserializationInterceptorContextRef, Intercept},
            runtime_components::RuntimeComponents,
        },
    };
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    #[derive(Debug)]
    struct Observe(Arc<AtomicUsize>);
    impl Intercept for Observe {
        fn name(&self) -> &'static str {
            "UserResponseObserver"
        }
        fn read_after_deserialization(
            &self,
            _: &AfterDeserializationInterceptorContextRef<'_>,
            _: &RuntimeComponents,
            _: &mut aws_smithy_types::config_bag::ConfigBag,
        ) -> Result<(), BoxError> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }
    let (endpoint, server) = serve(vec![
        (200, json!({})),
        (200, json!({"Count": 0, "ScannedCount": 0})),
    ]);
    let calls = Arc::new(AtomicUsize::new(0));
    let config = sdk_client(endpoint)
        .config()
        .to_builder()
        .interceptor(Observe(calls.clone()))
        .build();
    let client = OperationalCompat::client_with(Client::from_conf(config));
    assert_eq!(client.scan().run().await.unwrap().count, None);
    let condition = OperationalCompat::key_condition(OperationalCompat::id()).eq("id");
    assert_eq!(
        client
            .query()
            .key_condition(condition)
            .run()
            .await
            .unwrap()
            .count,
        Some(0)
    );
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert_eq!(server.join().unwrap().len(), 2);
}

// Deliberately populate every current SDK input field (including legacy ones).
// This checks forwarding, not whether DynamoDB accepts conflicting expressions.
#[cfg(feature = "aws-sdk")]
#[tokio::test]
async fn counter_interceptor_preserves_every_query_and_scan_input_field() {
    use aws_sdk::{
        types::{AttributeValue, ConditionalOperator, ReturnConsumedCapacity, Select},
        ResponseCounters,
    };
    let (endpoint, server) = serve(vec![(200, json!({})); 4]);
    let client = sdk_client(endpoint);
    let names = || {
        Some(std::collections::HashMap::from([(
            "#id".into(),
            "id".into(),
        )]))
    };
    let values = || {
        Some(std::collections::HashMap::from([(
            "id".into(),
            AttributeValue::S("id".into()),
        )]))
    };
    let query = aws_sdk::operation::query::QueryInput::builder()
        .table_name("table")
        .index_name("index")
        .select(Select::Count)
        .attributes_to_get("id")
        .limit(3)
        .consistent_read(true)
        .set_key_conditions(Some(Default::default()))
        .set_query_filter(Some(Default::default()))
        .conditional_operator(ConditionalOperator::And)
        .scan_index_forward(false)
        .set_exclusive_start_key(values())
        .return_consumed_capacity(ReturnConsumedCapacity::Total)
        .projection_expression("#id")
        .filter_expression("filter")
        .key_condition_expression("key")
        .set_expression_attribute_names(names())
        .set_expression_attribute_values(values());
    query.clone().send_with(&client).await.unwrap();
    ResponseCounters::query(&client, query, ResponseCounters::default())
        .await
        .unwrap();
    let scan = aws_sdk::operation::scan::ScanInput::builder()
        .table_name("table")
        .index_name("index")
        .select(Select::Count)
        .attributes_to_get("id")
        .limit(3)
        .consistent_read(true)
        .set_scan_filter(Some(Default::default()))
        .conditional_operator(ConditionalOperator::And)
        .set_exclusive_start_key(values())
        .return_consumed_capacity(ReturnConsumedCapacity::Total)
        .total_segments(2)
        .segment(1)
        .projection_expression("#id")
        .filter_expression("filter")
        .set_expression_attribute_names(names())
        .set_expression_attribute_values(values());
    scan.clone().send_with(&client).await.unwrap();
    ResponseCounters::scan(&client, scan, ResponseCounters::default())
        .await
        .unwrap();
    let requests = server.join().unwrap();
    assert_eq!(requests[0], requests[1]);
    assert_eq!(requests[2], requests[3]);
}
