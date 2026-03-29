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
    #[raiden(gsi = "testGSI")]
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

    #[tokio::test]
    async fn test_scan_index_limit_5() {
        let client = crate::all::create_client_from_struct!(Test);
        let res = client.scan().test_gsi().limit(5).run().await;

        assert_eq!(res.unwrap().items.len(), 5);
    }

    #[tokio::test]
    async fn test_scan_builder_keeps_deprecated_index_compatibility() {
        let client = crate::all::create_client_from_struct!(Test);
        #[allow(deprecated)]
        let _builder = client.scan().index("testGSI").limit(5);
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

    #[derive(Raiden)]
    #[raiden(table_name = "LastEvaluateKeyData")]
    #[raiden(gsi(name = "testGSI", partition_key = "ref_id"))]
    #[allow(dead_code)]
    pub struct TypedGsiProjectionScanSource {
        #[raiden(partition_key)]
        id: String,
        ref_id: String,
        long_text: String,
        #[raiden(omit_gsi = "testGSI")]
        omitted: String,
    }

    #[derive(RaidenIndex, Debug, PartialEq)]
    #[raiden(source = "TypedGsiProjectionScanSource", gsi = "testGSI")]
    #[allow(dead_code)]
    pub struct TypedGsiProjectionScanItem {
        ref_id: String,
        long_text: String,
    }

    #[tokio::test]
    async fn test_scan_with_renamed() {
        let client = crate::all::create_client_from_struct!(Project);
        let res = client.scan().limit(11).run().await;

        assert_eq!(res.unwrap().items.len(), 10);
    }

    #[tokio::test]
    async fn test_typed_gsi_scan_keeps_full_projection_without_projection_type() {
        let client = crate::all::create_client_from_struct!(TypedGsiProjectionScanSource);
        let builder = client.scan().test_gsi();

        #[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
        {
            let projection = builder
                .input
                .projection_expression
                .clone()
                .expect("projection should exist");
            let names = builder
                .input
                .expression_attribute_names
                .clone()
                .expect("attribute names should exist");

            assert!(projection.contains("#id"));
            assert!(projection.contains("#ref_id"));
            assert!(projection.contains("#long_text"));
            assert!(projection.contains("#omitted"));
            assert!(names.contains_key("#id"));
            assert!(names.contains_key("#ref_id"));
            assert!(names.contains_key("#long_text"));
            assert!(names.contains_key("#omitted"));
        }

        #[cfg(feature = "aws-sdk")]
        {
            let projection = builder
                .builder
                .get_projection_expression()
                .clone()
                .expect("projection should exist");
            let names = builder
                .builder
                .get_expression_attribute_names()
                .clone()
                .expect("attribute names should exist");

            assert!(projection.contains("#id"));
            assert!(projection.contains("#ref_id"));
            assert!(projection.contains("#long_text"));
            assert!(projection.contains("#omitted"));
            assert!(names.contains_key("#id"));
            assert!(names.contains_key("#ref_id"));
            assert!(names.contains_key("#long_text"));
            assert!(names.contains_key("#omitted"));
        }
    }

    #[tokio::test]
    async fn test_typed_gsi_scan_project_sets_projection_from_index_type() {
        let client = crate::all::create_client_from_struct!(TypedGsiProjectionScanSource);
        let builder = client.scan().test_gsi().project::<TypedGsiProjectionScanItem>();

        #[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
        {
            let projection = builder
                .inner
                .input
                .projection_expression
                .clone()
                .expect("projection should exist");
            let names = builder
                .inner
                .input
                .expression_attribute_names
                .clone()
                .expect("attribute names should exist");
            assert!(projection.contains("#ref_id"));
            assert!(projection.contains("#long_text"));
            assert!(!projection.contains("#id"));
            assert!(!names.contains_key("#id"));
            assert!(names.contains_key("#ref_id"));
            assert!(names.contains_key("#long_text"));
            assert!(!names.contains_key("#omitted"));
        }

        #[cfg(feature = "aws-sdk")]
        {
            let projection = builder
                .inner
                .builder
                .get_projection_expression()
                .clone()
                .expect("projection should exist");
            let names = builder
                .inner
                .builder
                .get_expression_attribute_names()
                .clone()
                .expect("attribute names should exist");
            assert!(projection.contains("#ref_id"));
            assert!(projection.contains("#long_text"));
            assert!(!projection.contains("#id"));
            assert!(!names.contains_key("#id"));
            assert!(names.contains_key("#ref_id"));
            assert!(names.contains_key("#long_text"));
            assert!(!names.contains_key("#omitted"));
        }
    }

    #[tokio::test]
    async fn test_typed_gsi_scan_run_with_accepts_index_type() {
        let client = crate::all::create_client_from_struct!(TypedGsiProjectionScanSource);
        let _future = client.scan().test_gsi().run_with::<TypedGsiProjectionScanItem>();
    }

    #[tokio::test]
    async fn test_typed_gsi_scan_project_can_chain_into_run_with() {
        fn assert_future_type<F>(_: F)
        where
            F: std::future::Future<
                    Output = Result<scan::ScanOutput<TypedGsiProjectionScanItem>, RaidenError>,
                >,
        {
        }

        let client = crate::all::create_client_from_struct!(TypedGsiProjectionScanSource);
        assert_future_type(
            client
                .scan()
                .test_gsi()
                .project::<TypedGsiProjectionScanItem>()
                .run_with::<TypedGsiProjectionScanItem>(),
        );
    }

    #[tokio::test]
    async fn test_typed_gsi_scan_project_run_returns_projection_output_type() {
        fn assert_future_type<F>(_: F)
        where
            F: std::future::Future<
                    Output = Result<scan::ScanOutput<TypedGsiProjectionScanItem>, RaidenError>,
                >,
        {
        }

        let client = crate::all::create_client_from_struct!(TypedGsiProjectionScanSource);
        assert_future_type(client.scan().test_gsi().project::<TypedGsiProjectionScanItem>().run());
    }

    #[tokio::test]
    async fn test_typed_gsi_scan_project_preserves_output_type_after_filter() {
        fn assert_future_type<F>(_: F)
        where
            F: std::future::Future<
                    Output = Result<scan::ScanOutput<TypedGsiProjectionScanItem>, RaidenError>,
                >,
        {
        }

        let client = crate::all::create_client_from_struct!(TypedGsiProjectionScanSource);
        let filter = TypedGsiProjectionScanSource::filter_expression(TypedGsiProjectionScanSource::ref_id())
            .eq("id0");
        assert_future_type(
            client
                .scan()
                .test_gsi()
                .project::<TypedGsiProjectionScanItem>()
                .filter(filter)
                .run(),
        );
    }

    #[tokio::test]
    async fn test_projection_item_scan_starts_typed_scan_builder() {
        fn assert_future_type<F>(_: F)
        where
            F: std::future::Future<
                    Output = Result<scan::ScanOutput<TypedGsiProjectionScanItem>, RaidenError>,
                >,
        {
        }

        let client = crate::all::create_client_from_struct!(TypedGsiProjectionScanSource);
        assert_future_type(TypedGsiProjectionScanItem::scan(&client).run());
    }

    #[tokio::test]
    async fn test_projection_item_scan_keeps_output_type_after_filter() {
        fn assert_future_type<F>(_: F)
        where
            F: std::future::Future<
                    Output = Result<scan::ScanOutput<TypedGsiProjectionScanItem>, RaidenError>,
                >,
        {
        }

        let client = crate::all::create_client_from_struct!(TypedGsiProjectionScanSource);
        let filter = TypedGsiProjectionScanSource::filter_expression(TypedGsiProjectionScanSource::ref_id())
            .eq("id0");
        assert_future_type(
            TypedGsiProjectionScanItem::scan(&client)
                .filter(filter)
                .run(),
        );
    }

    #[tokio::test]
    async fn test_deprecated_index_scan_keeps_full_projection() {
        let client = crate::all::create_client_from_struct!(TypedGsiProjectionScanSource);
        #[allow(deprecated)]
        let builder = client.scan().index("testGSI");

        #[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
        {
            let names = builder
                .input
                .expression_attribute_names
                .clone()
                .expect("attribute names should exist");
            assert!(names.contains_key("#omitted"));
        }

        #[cfg(feature = "aws-sdk")]
        {
            let names = builder
                .builder
                .get_expression_attribute_names()
                .clone()
                .expect("attribute names should exist");
            assert!(names.contains_key("#omitted"));
        }
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
