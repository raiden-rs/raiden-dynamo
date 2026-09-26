use raiden::*;

#[derive(Raiden)]
struct User {
    #[raiden(partition_key)]
    id: String,
    #[raiden(sort_key)]
    created_at: String,
}

fn main() {
    let _ = User::partition_key_condition()
        .eq("id")
        .and(User::sort_key_condition().eq("2026"))
        .and(User::sort_key_condition().eq("2027"));
}
