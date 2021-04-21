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

    #[test]
    fn test_scan() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = ScanTestData0::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
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
            )
        }
        rt.block_on(example());
    }

    #[derive(Raiden)]
    #[raiden(table_name = "LastEvaluateKeyData")]
    pub struct Test {
        #[raiden(partition_key)]
        id: String,
        ref_id: String,
        long_text: String,
    }

    #[test]
    fn test_scan_limit_1() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = Test::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let res = client.scan().limit(1).run().await;
            assert_eq!(res.unwrap().items.len(), 1);
        }
        rt.block_on(example());
    }

    #[test]
    fn test_scan_limit_5() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = Test::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let res = client.scan().limit(5).run().await;
            assert_eq!(res.unwrap().items.len(), 5);
        }
        rt.block_on(example());
    }

    #[test]
    fn test_scan_no_limit() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = Test::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let res = client.scan().run().await;
            assert_eq!(res.unwrap().items.len(), 10);
        }
        rt.block_on(example());
    }

    #[test]
    fn test_scan_over_limit() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = Test::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let res = client.scan().limit(11).run().await;
            assert_eq!(res.unwrap().items.len(), 10);
        }
        rt.block_on(example());
    }

    #[derive(Raiden)]
    #[raiden(table_name = "Project")]
    #[raiden(rename_all = "camelCase")]
    pub struct Project {
        #[raiden(partition_key)]
        id: String,
        org_id: String,
        updated_at: String,
    }

    #[test]
    fn test_scan_with_renamed() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = Project::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let res = client.scan().limit(11).run().await;
            assert_eq!(res.unwrap().items.len(), 10);
        }
        rt.block_on(example());
    }

    #[derive(Raiden, Debug, PartialEq)]
    #[raiden(table_name = "ScanTestData0")]
    pub struct ScanTestData0a {
        #[raiden(partition_key)]
        id: String,
        name: String,
    }

    #[test]
    fn test_scan_for_projection_expression() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = ScanTestData0a::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
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
            )
        }
        rt.block_on(example());
    }

    #[derive(Raiden, Debug, PartialEq)]
    #[raiden(table_name = "ScanLargeDataTest")]
    pub struct ScanLargeDataTest {
        #[raiden(partition_key)]
        id: String,
        ref_id: String,
        name: String,
    }

    #[test]
    fn should_be_scan_when_the_size_is_1mb_or_larger() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = ScanLargeDataTest::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let res = client.scan().run().await;
            assert_eq!(res.unwrap().items.len(), 100)
        }
        rt.block_on(example());
    }
}
