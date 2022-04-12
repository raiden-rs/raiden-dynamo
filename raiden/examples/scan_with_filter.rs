use raiden::*;

#[derive(Raiden, Debug)]
#[raiden(table_name = "ScanWithFilterTestData0")]
#[allow(dead_code)]
pub struct Scan {
    #[raiden(partition_key)]
    id: String,
    name: String,
    year: usize,
    num: usize,
}

#[tokio::main]
async fn main() {
    let client = Scan::client(Region::Custom {
        endpoint: "http://localhost:8000".into(),
        name: "ap-northeast-1".into(),
    });
    let filter = Scan::filter_expression(Scan::num()).eq(1000);
    let res = client.scan().filter(filter).run().await.unwrap();
    assert_eq!(res.items.len(), 50);
}
