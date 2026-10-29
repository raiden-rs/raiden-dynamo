// Generated operations return RaidenError, which includes SDK error variants.
#![allow(clippy::result_large_err)]

use raiden::*;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::time::{Duration, Instant};

#[derive(Raiden, Debug)]
pub struct BatchRead {
    #[raiden(partition_key)]
    pub id: String,
}

#[derive(Raiden, Debug)]
pub struct CompositeBatchRead {
    #[raiden(partition_key)]
    pub id: String,
    #[raiden(sort_key)]
    pub sequence: usize,
}

// Inspect actual requests, including the second batch and unprocessed-key retry.
fn serve_batches(
    table: &'static str,
    consistent: bool,
    composite: bool,
) -> (String, std::thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let endpoint = format!("http://{}", listener.local_addr().unwrap());
    let server = std::thread::spawn(move || {
        let mut pending = None;
        for expected_count in [100, 2, 1] {
            let deadline = Instant::now() + Duration::from_secs(10);
            let mut stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(Instant::now() < deadline, "missing batch_get request");
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    Err(error) => panic!("failed to accept batch_get request: {error}"),
                }
            };
            stream
                .set_read_timeout(Some(Duration::from_secs(10)))
                .unwrap();
            let mut reader = BufReader::new(&mut stream);
            let mut content_length = None;
            loop {
                let mut line = String::new();
                assert_ne!(reader.read_line(&mut line).unwrap(), 0);
                if line == "\r\n" {
                    break;
                }
                if let Some(length) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                    content_length = Some(length.trim().parse::<usize>().unwrap());
                }
            }
            let mut body = vec![0; content_length.unwrap()];
            reader.read_exact(&mut body).unwrap();
            let request: Value = serde_json::from_slice(&body).unwrap();
            let attributes = &request["RequestItems"][table];
            assert_eq!(
                attributes.get("ConsistentRead"),
                consistent.then_some(&Value::Bool(true))
            );
            let keys = attributes["Keys"].as_array().unwrap();
            assert_eq!(keys.len(), expected_count);
            if let Some(key) = pending.take() {
                assert_eq!(keys[0], key);
            }
            for key in keys {
                assert!(key["id"]["S"].is_string());
                assert_eq!(key.get("sequence").is_some(), composite);
            }
            let mut response = json!({"Responses": {table: []}});
            if expected_count > 1 {
                let key = keys.last().unwrap().clone();
                response["UnprocessedKeys"] = json!({table: {"Keys": [key.clone()]}});
                pending = Some(key);
            }
            let body = response.to_string();
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/x-amz-json-1.0\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(), body
            )
            .unwrap();
        }
    });
    (endpoint, server)
}

#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
fn sdk_client(endpoint: String) -> (Client, Region) {
    let region = Region::Custom {
        endpoint,
        name: "ap-northeast-1".into(),
    };
    let client = Client::new_with(
        StaticProvider::new_minimal("dummy".into(), "dummy".into()),
        HttpClient::new().unwrap(),
    );
    (client, region)
}

#[cfg(feature = "aws-sdk")]
fn sdk_client(endpoint: String) -> Client {
    let config = aws_sdk::config::Config::builder()
        .behavior_version(aws_sdk::config::BehaviorVersion::latest())
        .region(config::Region::new("ap-northeast-1"))
        .endpoint_url(endpoint)
        .credentials_provider(aws_credential_types::Credentials::new(
            "dummy", "dummy", None, None, "test",
        ))
        .build();
    Client::from_conf(config)
}

macro_rules! batch_client {
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

#[tokio::test]
async fn batch_get_consistency_across_batches_and_retries() {
    for consistent in [false, true] {
        let (endpoint, server) = serve_batches("BatchRead", consistent, false);
        let client = batch_client!(BatchRead, endpoint);
        let builder = client.batch_get((0..101).map(|id| id.to_string()).collect());
        let builder = if consistent {
            builder.consistent()
        } else {
            builder
        };
        let result = builder.run().await.unwrap();
        assert!(result.items.is_empty());
        server.join().unwrap();
    }
}

#[tokio::test]
async fn composite_batch_get_consistency_across_batches_and_retries() {
    for consistent in [false, true] {
        let (endpoint, server) = serve_batches("CompositeBatchRead", consistent, true);
        let client = batch_client!(CompositeBatchRead, endpoint);
        let builder = client.batch_get((0..101usize).map(|id| (id.to_string(), id)).collect());
        let builder = if consistent {
            builder.consistent()
        } else {
            builder
        };
        let result = builder.run().await.unwrap();
        assert!(result.items.is_empty());
        server.join().unwrap();
    }
}
