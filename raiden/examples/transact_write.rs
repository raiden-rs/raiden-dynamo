use raiden::*;

#[derive(Raiden)]
#[raiden(table_name = "user")]
pub struct User {
    #[raiden(partition_key)]
    id: String,
    name: String,
}

fn main() {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    async fn example() {
        let mut tx = ::raiden::WriteTx::new(Region::Custom {
            endpoint: "http://localhost:8000".into(),
            name: "ap-northeast-1".into(),
        });
        let input = User::put_item_builder()
            .id("testId".to_owned())
            .name("bokuweb".to_owned())
            .build()
            .unwrap();
        let input2 = User::put_item_builder()
            .id("testId2".to_owned())
            .name("bokuweb".to_owned())
            .build()
            .unwrap();
        tx.put(User::put(input))
            .put(User::put(input2))
            .run()
            .await
            .unwrap();
    }
    rt.block_on(example());
}
