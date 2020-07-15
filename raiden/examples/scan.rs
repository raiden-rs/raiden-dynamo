use raiden::*;

#[derive(Raiden, Debug)]
#[raiden(table_name = "ScanTestData0")]
pub struct ScanTestData0 {
    #[raiden(partition_key)]
    id: String,
    name: String,
    year: usize,
    num: usize,
}

fn main() {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    async fn example() {
        let client = ScanTestData0::client(Region::Custom {
            endpoint: "http://localhost:8000".into(),
            name: "ap-northeast-1".into(),
        });
        let res = client.scan().run().await;
        dbg!(&res);
    }
    rt.block_on(example());
}
