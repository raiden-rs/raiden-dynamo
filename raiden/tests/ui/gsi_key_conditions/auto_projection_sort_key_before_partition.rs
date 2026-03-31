use raiden::*;

#[derive(Raiden)]
#[raiden(table_name = "LastEvaluateKeyData")]
#[raiden(
    gsi(
        name = "userIndex",
        partition_key = "ref_id",
        sort_key = "id",
        sort_key = "long_text"
    )
)]
struct User {
    #[raiden(partition_key)]
    id: String,
    ref_id: String,
    long_text: String,
    #[raiden(omit_gsi = "userIndex")]
    omitted: String,
}

fn create_client() -> UserClient {
    unimplemented!()
}

fn main() {
    let client = create_client();

    let cond = UserIndexItem::id().eq("id1");
    let _ = client.query().user_index().key_condition(cond);
}
