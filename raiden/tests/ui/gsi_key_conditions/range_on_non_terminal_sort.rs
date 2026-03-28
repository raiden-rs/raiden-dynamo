use raiden::*;

#[derive(Raiden)]
#[raiden(table_name = "LastEvaluateKeyData")]
#[raiden(
    gsi(
        name = "testGSI",
        partition_key = "ref_id",
        sort_key = "id",
        sort_key = "long_text"
    )
)]
struct TypedCompositeGsiSortKeyTest {
    #[raiden(partition_key)]
    id: String,
    ref_id: String,
    long_text: String,
}

fn main() {
    let _ = TypedCompositeGsiSortKeyTest::test_gsi_sort_key_condition_1().between("id0", "id9");
}
