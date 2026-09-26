use raiden::*;

#[derive(Raiden)]
struct User {
    #[raiden(partition_key)]
    id: String,
    #[raiden(sort_key)]
    created_at: String,
}

fn client() -> UserClient {
    unimplemented!()
}

fn main() {
    let _ = client()
        .query()
        .key_condition(User::sort_key_condition().eq("2026"));
}
