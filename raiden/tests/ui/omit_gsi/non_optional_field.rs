use raiden::*;

#[derive(Raiden)]
#[raiden(table_name = "Users")]
#[raiden(gsi(name = "userIndex", partition_key = "user_id"))]
struct InvalidOmitGsiField {
    #[raiden(partition_key)]
    id: String,
    user_id: String,
    #[raiden(omit_gsi = "userIndex")]
    name: String,
}

fn main() {}
