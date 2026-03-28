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

#[derive(RaidenIndex)]
#[raiden(source = "TypedCompositeGsiSortKeyTest", gsi = "testGSI")]
#[raiden(
    gsi(
        name = "testGSI",
        partition_key = "ref_id",
        sort_key = "id",
        sort_key = "long_text"
    )
)]
struct TypedCompositeGsiProjectionItem {
    ref_id: String,
    id: String,
    long_text: String,
}

fn main() {
    let client = TypedCompositeGsiSortKeyTest::client(Region::Custom {
        endpoint: "http://localhost:8000".into(),
        name: "ap-northeast-1".into(),
    });

    let cond = TypedCompositeGsiProjectionItem::test_gsi_sort_key_condition_1().eq("id1");
    let _ = client.query().test_gsi().key_condition(cond);
}
