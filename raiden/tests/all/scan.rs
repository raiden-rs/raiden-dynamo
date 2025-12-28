#[cfg(test)]
mod tests {

    #[cfg(test)]
    use pretty_assertions::assert_eq;
    use raiden::*;

    #[derive(Raiden, Debug, PartialEq)]
    pub struct ScanTestData0 {
        #[raiden(partition_key)]
        id: String,
        name: String,
        year: usize,
        num: usize,
    }

    #[tokio::test]
    async fn test_scan() {
        let client = crate::all::create_client_from_struct!(ScanTestData0);
        let res = client.scan().run().await;

        assert_eq!(
            res.unwrap(),
            scan::ScanOutput {
                consumed_capacity: None,
                count: Some(1),
                items: vec![ScanTestData0 {
                    id: "scanId0".to_owned(),
                    name: "scanAlice".to_owned(),
                    year: 2001,
                    num: 2000
                }],
                last_evaluated_key: None,
                scanned_count: Some(1),
            }
        );
    }

    #[derive(Raiden)]
    #[raiden(table_name = "LastEvaluateKeyData")]
    #[allow(dead_code)]
    pub struct Test {
        #[raiden(partition_key)]
        id: String,
        ref_id: String,
        long_text: String,
    }

    #[tokio::test]
    async fn test_scan_limit_1() {
        let client = crate::all::create_client_from_struct!(Test);
        let res = client.scan().limit(1).run().await;

        assert_eq!(res.unwrap().items.len(), 1);
    }

    #[tokio::test]
    async fn test_scan_limit_5() {
        let client = crate::all::create_client_from_struct!(Test);
        let res = client.scan().limit(5).run().await;

        assert_eq!(res.unwrap().items.len(), 5);
    }

    #[tokio::test]
    async fn test_scan_no_limit() {
        let client = crate::all::create_client_from_struct!(Test);
        let res = client.scan().run().await;

        assert_eq!(res.unwrap().items.len(), 10);
    }

    #[tokio::test]
    async fn test_scan_over_limit() {
        let client = crate::all::create_client_from_struct!(Test);
        let res = client.scan().limit(11).run().await;

        assert_eq!(res.unwrap().items.len(), 10);
    }

    #[derive(Raiden)]
    #[raiden(table_name = "Project")]
    #[raiden(rename_all = "camelCase")]
    #[allow(dead_code)]
    pub struct Project {
        #[raiden(partition_key)]
        id: String,
        org_id: String,
        updated_at: String,
    }

    #[tokio::test]
    async fn test_scan_with_renamed() {
        let client = crate::all::create_client_from_struct!(Project);
        let res = client.scan().limit(11).run().await;

        assert_eq!(res.unwrap().items.len(), 10);
    }

    #[derive(Raiden, Debug, PartialEq)]
    #[raiden(table_name = "ScanTestData0")]
    pub struct ScanTestData0a {
        #[raiden(partition_key)]
        id: String,
        name: String,
    }

    #[tokio::test]
    async fn test_scan_for_projection_expression() {
        let client = crate::all::create_client_from_struct!(ScanTestData0a);
        let res = client.scan().run().await;

        assert_eq!(
            res.unwrap(),
            scan::ScanOutput {
                consumed_capacity: None,
                count: Some(1),
                items: vec![ScanTestData0a {
                    id: "scanId0".to_owned(),
                    name: "scanAlice".to_owned(),
                }],
                last_evaluated_key: None,
                scanned_count: Some(1),
            }
        );
    }

    #[derive(Raiden, Debug, PartialEq)]
    #[raiden(table_name = "ScanLargeDataTest")]
    pub struct ScanLargeDataTest {
        #[raiden(partition_key)]
        id: String,
        ref_id: String,
        name: String,
    }

    #[tokio::test]
    async fn should_be_scan_when_the_size_is_1mb_or_larger() {
        let client = crate::all::create_client_from_struct!(ScanLargeDataTest);
        let res = client.scan().run().await;

        assert_eq!(res.unwrap().items.len(), 100);
    }

    #[derive(Raiden, Debug)]
    #[raiden(table_name = "ScanWithFilterTestData0")]
    #[allow(dead_code)]
    pub struct Scan {
        #[raiden(partition_key)]
        id: String,
        name: String,
        year: usize,
        num: usize,
        option: Option<String>,
    }

    #[tokio::test]
    async fn test_simple_filter() {
        let client = crate::all::create_client_from_struct!(Scan);
        let filter = Scan::filter_expression(Scan::num()).eq(1000);
        let res = client.scan().filter(filter).run().await.unwrap();

        assert_eq!(res.items.len(), 50);
    }

    #[tokio::test]
    async fn test_size_filter() {
        let client = crate::all::create_client_from_struct!(Scan);
        let filter = Scan::filter_expression(Scan::name()).size().eq(10);
        let res = client.scan().filter(filter).run().await.unwrap();

        assert_eq!(res.items.len(), 10);
    }

    #[tokio::test]
    async fn test_or_with_contain_filter() {
        let client = crate::all::create_client_from_struct!(Scan);
        let filter = Scan::filter_expression(Scan::num())
            .eq(1000)
            .or(Scan::filter_expression(Scan::id()).contains("scanId50"));
        let res = client.scan().filter(filter).run().await.unwrap();

        assert_eq!(res.items.len(), 51);
    }

    #[tokio::test]
    async fn test_attribute_exists_filter() {
        let client = crate::all::create_client_from_struct!(Scan);
        let filter = Scan::filter_expression(Scan::option()).attribute_exists();
        let res = client.scan().filter(filter).run().await.unwrap();

        assert_eq!(res.items.len(), 50);
    }

    #[tokio::test]
    async fn test_attribute_not_exists_filter() {
        let client = crate::all::create_client_from_struct!(Scan);
        let filter = Scan::filter_expression(Scan::option()).attribute_not_exists();
        let res = client.scan().filter(filter).run().await.unwrap();

        assert_eq!(res.items.len(), 50);
    }

    #[tokio::test]
    async fn test_attribute_type_filter() {
        let client = crate::all::create_client_from_struct!(Scan);
        let filter =
            Scan::filter_expression(Scan::option()).attribute_type(raiden::AttributeType::S);
        let res = client.scan().filter(filter).run().await.unwrap();

        assert_eq!(res.items.len(), 50);
    }
}
