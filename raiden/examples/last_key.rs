use raiden::*;

#[derive(Raiden)]
#[raiden(table_name = "LastEvaluateKeyData")]
pub struct Test {
    #[raiden(partition_key)]
    id: String,
    ref_id: String,
    long_text: String,
}

fn main() {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    async fn example() {
        let client = Test::client(Region::Custom {
            endpoint: "http://localhost:8000".into(),
            name: "ap-northeast-1".into(),
        });
        let cond = Test::key_condition(Test::ref_id()).eq("id0");
        let res = client
            .query()
            .index("testGSI")
            .limit(5)
            .key_condition(cond)
            .run()
            .await;
        dbg!(&res.unwrap().items.len());
    }
    rt.block_on(example());
}
