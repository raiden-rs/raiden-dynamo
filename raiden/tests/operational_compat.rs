//! Identical scripted DynamoDB responses on both backends, without AWS access.
//! SDK retries are disabled: retry tests exercise Raiden's explicit policy only.
#![allow(clippy::result_large_err)]
use raiden::*;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::time::{Duration, Instant};

#[derive(Raiden, Debug, PartialEq)]
pub struct OperationalCompat {
    #[raiden(partition_key)]
    pub id: String,
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
