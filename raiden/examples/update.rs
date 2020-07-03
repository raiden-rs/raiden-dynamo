use raiden::*;

#[derive(Raiden)]
#[raiden(table_name = "UpdateTestData0")]
pub struct Example {
    #[raiden(partition_key)]
    id: String,
    name: String,
    age: u8,
}

fn main() {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    async fn example() {
        let client = Example::client(Region::Custom {
            endpoint: "http://localhost:8000".into(),
            name: "ap-northeast-1".into(),
        });
        let set_expression = Example::update_expression()
            .set(Example::name())
            .value("updated!!");
        let res = client.update("id0").set(set_expression).run().await;
        dbg!(res);
    }
    rt.block_on(example());
}
