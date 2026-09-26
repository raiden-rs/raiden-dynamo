use raiden::*;

#[derive(Raiden)]
struct User {
    #[raiden(partition_key)]
    id: String,
    #[raiden(sort_key)]
    created_at: String,
    name: String,
}

fn main() {
    let _ = User::partition_key_condition()
        .eq("id")
        .and(User::key_condition(User::name()).eq("name"));
}
