use raiden::*;

#[derive(Raiden)]
#[raiden(table_name = "user")]
pub struct User {
    #[raiden(partition_key)]
    pub id: String,
}

struct MyRetryStrategy;

impl RetryStrategy for MyRetryStrategy {
    fn should_retry(&self, _error: &RaidenError, request_count: usize) -> bool {
        log::info!("request count {}", request_count);
        true
    }

    fn policy(&self) -> Policy {
        Policy::Limit(3)
    }
}

fn main() {
    std::env::set_var("RUST_LOG", "info");
    pretty_env_logger::init();
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    async fn example() {
        let client = User::client(Region::Custom {
            endpoint: "http://localhost:8000".into(),
            name: "ap-northeast-1".into(),
        });
        let _ = client
            .with_retries(Box::new(MyRetryStrategy))
            .get("anonymous")
            .run()
            .await;
    }
    rt.block_on(example());
}
