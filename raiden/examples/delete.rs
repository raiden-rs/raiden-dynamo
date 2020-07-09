use raiden::*;

#[derive(Raiden)]
#[raiden(table_name = "QueryTestData0")]
#[derive(Debug, Clone)]
pub struct Test {
    #[raiden(partition_key)]
    id: String,
    name: String,
    #[raiden(sort_key)]
    year: usize,
}

fn main() {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    async fn example() {
        let client = Test::client(Region::Custom {
            endpoint: "http://localhost:8000".into(),
            name: "ap-northeast-1".into(),
        });

        let res = client.delete("id1", 2003).run().await;
        dbg!(&res);
    }
    rt.block_on(example());
}
