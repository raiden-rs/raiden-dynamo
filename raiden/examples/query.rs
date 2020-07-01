use raiden::*;

#[derive(Raiden, Debug)]
#[raiden(table_name = "QueryTestData0")]
pub struct QueryTestData0 {
    #[raiden(partition_key)]
    id: String,
    name: String,
    year: usize,
    num: usize,
}

fn main() {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    async fn example() {
        let client = QueryTestData0::client(Region::Custom {
            endpoint: "http://localhost:8000".into(),
            name: "ap-northeast-1".into(),
        });
        let cond = QueryTestData0::key_condition(QueryTestData0AttrNames::Id)
            .eq("id0")
            .and(QueryTestData0::key_condition(QueryTestData0AttrNames::Year).eq(1999));
        let res = client.query().key_condition(cond).run().await;
        dbg!(&res);
        let cond = QueryTestData0::key_condition(QueryTestData0AttrNames::Id)
            .eq("id0")
            .and(QueryTestData0::key_condition(QueryTestData0AttrNames::Year).eq(1999));
        let res = client.query().key_condition(cond).run().await;
        dbg!(&res);
    }
    rt.block_on(example());
}
