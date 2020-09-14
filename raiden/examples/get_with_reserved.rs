use raiden::*;

#[derive(Raiden)]
#[raiden(table_name = "ReservedTestData0")]
pub struct Reserved {
    #[raiden(partition_key)]
    id: String,
    #[raiden(rename = "type")]
    some_type: String,
}

fn main() {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    async fn example() {
        let mut client = User::client(Region::Custom {
            endpoint: "http://localhost:8000".into(),
            name: "ap-northeast-1".into(),
        });
        let _ = client.get("id0").run().await;
    }
    rt.block_on(example());
}
