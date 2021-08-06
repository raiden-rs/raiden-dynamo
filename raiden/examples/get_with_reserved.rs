use raiden::*;

#[derive(Raiden)]
#[raiden(table_name = "ReservedTestData0")]
pub struct Reserved {
    #[raiden(partition_key)]
    pub id: String,
    #[raiden(rename = "type")]
    pub some_type: String,
}

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    async fn example() {
        let client = Reserved::client(Region::Custom {
            endpoint: "http://localhost:8000".into(),
            name: "ap-northeast-1".into(),
        });
        let _ = client.get("id0").run().await;
    }
    rt.block_on(example());
}
