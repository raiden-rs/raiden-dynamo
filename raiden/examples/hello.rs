use raiden::*;

#[derive(Raiden)]
#[raiden(table_name = "user")]
pub struct User {
    #[raiden(partition_key)]
    id: String,
    #[raiden(uuid)]
    uuid: String,
    name: String,
}

fn main() {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    async fn example() {
        let client = User::client(Region::Custom {
            endpoint: "http://localhost:8000".into(),
            name: "ap-northeast-1".into(),
        });
        //let user = UserPutItemInput {
        //    id: "a".to_owned(),
        //    name: "bokuweb".to_owned(),
        //    // uuid: "aa".to_owned(),
        //};
        //let cond = User::condition()
        //    .attr(User::name())
        //    .eq_attr(User::name());
        //
        //// let cond = User::condition().not().attr_type(User::name(), AttributeType::N);
        //// .and(User::condition().not().attribute_exists(User::id()));
        let res = client
            .batch_get()
            .add_key("user_primary_key") /*.condition(cond)*/
            .run()
            .await;
    }
    rt.block_on(example());
}
