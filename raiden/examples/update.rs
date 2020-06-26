use raiden::*;

#[derive(Raiden)]
#[raiden(table_name = "UpdateTestData0")]
pub struct UpdateTestData0 {
    #[raiden(partition_key)]
    id: String,
    name: String,
}

fn main() {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    async fn example() {
        let client = UpdateTestData0::client(Region::Custom {
            endpoint: "http://localhost:8000".into(),
            name: "ap-northeast-1".into(),
        });
        let res = client
            .update("id0")
            .set(UpdateTestData0AttrNames::Name, "updated!!!!!!!!!!")
            .run()
            .await;
    }
    rt.block_on(example());
}
