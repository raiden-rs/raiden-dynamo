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
    let cond = TypedCompositeGsiSortKeyTest::test_gsi_key_condition()
        .eq("id0")
        .and(TypedCompositeGsiSortKeyTest::test_gsi_sort_key_condition_1().eq("id1"))
        .and(TypedCompositeGsiSortKeyTest::test_gsi_sort_key_condition_2().begins_with("long"));

    let _ = cond.and(TypedCompositeGsiSortKeyTest::test_gsi_sort_key_condition_1().eq("id2"));
}
