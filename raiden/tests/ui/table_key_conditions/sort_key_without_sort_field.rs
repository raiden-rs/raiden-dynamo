use raiden::*;

#[derive(Raiden)]
struct User {
    #[raiden(partition_key)]
    id: String,
}

fn main() {
    let _ = User::sort_key_condition().eq("2026");
}
