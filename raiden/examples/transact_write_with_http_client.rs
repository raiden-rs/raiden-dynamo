use raiden::*;
use tracing_subscriber::{
    fmt::{format::FmtSpan, time::UtcTime},
    EnvFilter,
};

#[derive(Raiden)]
#[raiden(table_name = "user")]
pub struct User {
    #[raiden(partition_key)]
    pub id: String,
    pub name: String,
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("raiden=debug,info"))
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(true)
        .with_timer(UtcTime::rfc_3339())
        .init();

    let rt = tokio::runtime::Runtime::new().unwrap();
    async fn example() {
        let dispatcher =
            raiden::request::HttpClient::new().expect("failed to create request dispatcher");
        let credentials_provider = raiden::credential::DefaultCredentialsProvider::new()
            .expect("failed to create credentials provider");
        let core_client = raiden::Client::new_with(credentials_provider, dispatcher);

        let tx = ::raiden::WriteTx::new_with_client(
            core_client,
            Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            },
        );
        let cond = User::condition().attr_not_exists(User::id());
        let input = User::put_item_builder()
            .id("testId".to_owned())
            .name("bokuweb".to_owned())
            .build();
        let input2 = User::put_item_builder()
            .id("testId2".to_owned())
            .name("bokuweb".to_owned())
            .build();
        tx.put(User::put(input).condition(cond))
            .put(User::put(input2))
            .run()
            .await
            .unwrap();
    }
    rt.block_on(example());
}
