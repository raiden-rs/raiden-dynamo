use raiden::*;
use tracing_subscriber::{
    fmt::{format::FmtSpan, time::UtcTime},
    EnvFilter,
};

#[derive(Raiden)]
#[raiden(table_name = "user")]
pub struct User {
    #[raiden(partition_key)]
    pub id: String,
    #[raiden(sort_key)]
    pub year: usize,
    #[raiden(uuid)]
    pub uuid: String,
    pub name: String,
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("hello=debug,info"))
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(true)
        .with_timer(UtcTime::rfc_3339())
        .init();

    let rt = tokio::runtime::Runtime::new().unwrap();
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
        let keys: Vec<(&str, usize)> = vec![("bokuweb", 2019), ("raiden", 2020)];
        let _ = client.batch_get(keys).run().await;
    }
    rt.block_on(example());
}
