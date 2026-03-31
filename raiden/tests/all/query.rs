#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    #[cfg(test)]
    use pretty_assertions::assert_eq;
    use raiden::*;

    #[derive(Raiden, Debug, PartialEq)]
    pub struct QueryTestData0 {
        #[raiden(partition_key)]
        id: String,
        name: String,
        year: usize,
        num: usize,
        option: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, RaidenDocument)]
    pub struct Profile {
        level: usize,
        nickname: String,
    }

    #[derive(Raiden)]
    #[raiden(table_name = "user")]
    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    pub struct QueryMapPathTest {
        #[raiden(partition_key)]
        id: String,
        profile: Profile,
        metadata: HashMap<String, usize>,
    }

    #[tokio::test]
    async fn test_query() {
        let client = crate::all::create_client_from_struct!(QueryTestData0);
        let cond = QueryTestData0::key_condition(QueryTestData0::id()).eq("id0");
        let res = client.query().key_condition(cond).run().await;

        assert_eq!(
            res.unwrap(),
            query::QueryOutput {
                consumed_capacity: None,
                count: Some(2),
                items: vec![
                    QueryTestData0 {
                        id: "id0".to_owned(),
                        name: "john".to_owned(),
                        year: 1999,
                        num: 1000,
                        option: None,
                    },
                    QueryTestData0 {
                        id: "id0".to_owned(),
                        name: "john".to_owned(),
                        year: 2000,
                        num: 2000,
                        option: None,
                    },
                ],
                next_token: None,
                scanned_count: Some(2),
            }
        );
    }

    #[tokio::test]
    async fn test_query_with_and_key_condition() {
        let client = crate::all::create_client_from_struct!(QueryTestData0);
        let cond = QueryTestData0::key_condition(QueryTestData0::id())
            .eq("id0")
            .and(QueryTestData0::key_condition(QueryTestData0::year()).eq(1999));
        let res = client.query().key_condition(cond).run().await;

        assert_eq!(
            res.unwrap(),
            query::QueryOutput {
                consumed_capacity: None,
                count: Some(1),
                items: vec![QueryTestData0 {
                    id: "id0".to_owned(),
                    name: "john".to_owned(),
                    year: 1999,
                    num: 1000,
                    option: None,
                },],
                next_token: None,
                scanned_count: Some(1),
            }
        );
    }

    #[tokio::test]
    async fn test_query_with_simple_filter() {
        let client = crate::all::create_client_from_struct!(QueryTestData0);
        let cond = QueryTestData0::key_condition(QueryTestData0::id()).eq("id3");
        let filter = QueryTestData0::filter_expression(QueryTestData0::num()).eq(4000);
        let res = client
            .query()
            .key_condition(cond)
            .filter(filter)
            .run()
            .await
            .unwrap();
        assert_eq!(res.items.len(), 3);
    }

    #[tokio::test]
    async fn test_query_builder_keeps_map_path_attribute_names() {
        reset_value_id();
        let client = crate::all::create_client_from_struct!(QueryMapPathTest);
        let cond = QueryMapPathTest::key_condition(QueryMapPathTest::id()).eq("id0");
        let filter =
            QueryMapPathTest::filter_expression(QueryMapPathTest::metadata().key("score")).eq(42);

        #[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
        let input = client.query().key_condition(cond).filter(filter).input;
        #[cfg(feature = "aws-sdk")]
        let input = client
            .query()
            .key_condition(cond)
            .filter(filter)
            .builder
            .build()
            .unwrap();

        let mut expected_names = std::collections::HashMap::new();
        expected_names.insert("#id".to_owned(), "id".to_owned());
        expected_names.insert("#profile".to_owned(), "profile".to_owned());
        expected_names.insert("#metadata".to_owned(), "metadata".to_owned());
        expected_names.insert("#score".to_owned(), "score".to_owned());

        #[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
        {
            assert!(input
                .filter_expression
                .as_ref()
                .is_some_and(|expr| expr.starts_with("#metadata.#score = :value")));
            assert_eq!(input.expression_attribute_names, Some(expected_names));
        }

        #[cfg(feature = "aws-sdk")]
        {
            assert!(input
                .filter_expression()
                .is_some_and(|expr| expr.starts_with("#metadata.#score = :value")));
            assert_eq!(input.expression_attribute_names(), Some(&expected_names));
        }
    }

    #[tokio::test]
    async fn test_query_builder_keeps_document_path_attribute_names() {
        reset_value_id();
        let client = crate::all::create_client_from_struct!(QueryMapPathTest);
        let cond = QueryMapPathTest::key_condition(QueryMapPathTest::id()).eq("id0");
        let filter = QueryMapPathTest::filter_expression(
            QueryMapPathTest::profile().field(Profile::level()),
        )
        .eq(3);

        #[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
        let input = client.query().key_condition(cond).filter(filter).input;
        #[cfg(feature = "aws-sdk")]
        let input = client
            .query()
            .key_condition(cond)
            .filter(filter)
            .builder
            .build()
            .unwrap();

        let mut expected_names = std::collections::HashMap::new();
        expected_names.insert("#id".to_owned(), "id".to_owned());
        expected_names.insert("#profile".to_owned(), "profile".to_owned());
        expected_names.insert("#metadata".to_owned(), "metadata".to_owned());
        expected_names.insert("#level".to_owned(), "level".to_owned());

        #[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
        {
            assert!(input
                .filter_expression
                .as_ref()
                .is_some_and(|expr| expr.starts_with("#profile.#level = :value")));
            assert_eq!(input.expression_attribute_names, Some(expected_names));
        }

        #[cfg(feature = "aws-sdk")]
        {
            assert!(input
                .filter_expression()
                .is_some_and(|expr| expr.starts_with("#profile.#level = :value")));
            assert_eq!(input.expression_attribute_names(), Some(&expected_names));
        }
    }

    #[tokio::test]
    async fn test_query_with_size_filter() {
        let client = crate::all::create_client_from_struct!(QueryTestData0);
        let cond = QueryTestData0::key_condition(QueryTestData0::id()).eq("id5");
        let filter = QueryTestData0::filter_expression(QueryTestData0::name())
            .size()
            .ge(4);
        let res = client
            .query()
            .key_condition(cond)
            .filter(filter)
            .run()
            .await
            .unwrap();
        assert_eq!(res.items.len(), 2);
    }

    #[tokio::test]
    async fn test_query_with_or_filter() {
        let client = crate::all::create_client_from_struct!(QueryTestData0);
        let cond = QueryTestData0::key_condition(QueryTestData0::id()).eq("id3");
        let filter = QueryTestData0::filter_expression(QueryTestData0::name())
            .eq("bar0")
            .or(QueryTestData0::filter_expression(QueryTestData0::name()).eq("bar1"));
        let res = client
            .query()
            .key_condition(cond)
            .filter(filter)
            .run()
            .await
            .unwrap();
        assert_eq!(res.items.len(), 2);
    }

    #[tokio::test]
    async fn test_query_with_attribute_exists_filter() {
        let client = crate::all::create_client_from_struct!(QueryTestData0);
        let cond = QueryTestData0::key_condition(QueryTestData0::id()).eq("id4");
        let filter = QueryTestData0::filter_expression(QueryTestData0::option()).attribute_exists();
        let res = client
            .query()
            .key_condition(cond)
            .filter(filter)
            .run()
            .await
            .unwrap();
        assert_eq!(res.items.len(), 2);
    }

    #[tokio::test]
    async fn test_query_with_attribute_not_exists_filter() {
        let client = crate::all::create_client_from_struct!(QueryTestData0);
        let cond = QueryTestData0::key_condition(QueryTestData0::id()).eq("id4");
        let filter =
            QueryTestData0::filter_expression(QueryTestData0::option()).attribute_not_exists();
        let res = client
            .query()
            .key_condition(cond)
            .filter(filter)
            .run()
            .await
            .unwrap();
        assert_eq!(res.items.len(), 1);
    }

    #[tokio::test]
    async fn test_query_with_attribute_type_filter() {
        let client = crate::all::create_client_from_struct!(QueryTestData0);
        let cond = QueryTestData0::key_condition(QueryTestData0::id()).eq("id4");
        let filter = QueryTestData0::filter_expression(QueryTestData0::option())
            .attribute_type(raiden::AttributeType::S);
        let res = client
            .query()
            .key_condition(cond)
            .filter(filter)
            .run()
            .await
            .unwrap();
        assert_eq!(res.items.len(), 2);
    }

    #[tokio::test]
    async fn test_query_with_contains_filter() {
        let client = crate::all::create_client_from_struct!(QueryTestData0);
        let cond = QueryTestData0::key_condition(QueryTestData0::id()).eq("id4");
        let filter = QueryTestData0::filter_expression(QueryTestData0::name()).contains("bar");
        let res = client
            .query()
            .key_condition(cond)
            .filter(filter)
            .run()
            .await
            .unwrap();
        assert_eq!(res.items.len(), 2);
    }

    #[tokio::test]
    async fn test_query_in_filter() {
        let client = crate::all::create_client_from_struct!(QueryTestData0);
        let cond = QueryTestData0::key_condition(QueryTestData0::id()).eq("id4");
        let filter =
            QueryTestData0::filter_expression(QueryTestData0::name()).r#in(vec!["bar0", "bar1"]);
        let res = client
            .query()
            .key_condition(cond)
            .filter(filter)
            .run()
            .await
            .unwrap();
        assert_eq!(res.items.len(), 2);
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

    #[derive(Raiden)]
    #[raiden(table_name = "LastEvaluateKeyData")]
    #[raiden(gsi(name = "testGSI", partition_key = "ref_id"))]
    #[allow(dead_code)]
    pub struct TypedGsiPartitionKeyTest {
        #[raiden(partition_key)]
        id: String,
        ref_id: String,
        long_text: String,
    }

    #[derive(Raiden)]
    #[raiden(table_name = "LastEvaluateKeyData")]
    #[raiden(gsi(name = "testGSI", partition_key = "ref_id", sort_key = "id"))]
    #[allow(dead_code)]
    pub struct TypedGsiSortKeyTest {
        #[raiden(partition_key)]
        id: String,
        ref_id: String,
        long_text: String,
    }

    #[derive(Raiden)]
    #[raiden(table_name = "LastEvaluateKeyData")]
    #[raiden(gsi(
        name = "testGSI",
        partition_key = "ref_id",
        sort_key = "id",
        sort_key = "long_text"
    ))]
    #[allow(dead_code)]
    pub struct TypedCompositeGsiSortKeyTest {
        #[raiden(partition_key)]
        id: String,
        ref_id: String,
        long_text: String,
    }

    #[derive(Raiden)]
    #[raiden(table_name = "LastEvaluateKeyData")]
    #[raiden(gsi(name = "testGSI", partition_key = "ref_id"))]
    #[allow(dead_code)]
    pub struct TypedGsiProjectionSource {
        #[raiden(partition_key)]
        id: String,
        ref_id: String,
        long_text: String,
        #[raiden(omit_gsi = "testGSI")]
        omitted: String,
    }

    #[derive(RaidenIndex, Debug, PartialEq)]
    #[raiden(source = "TypedGsiProjectionSource", gsi = "testGSI")]
    #[raiden(gsi(name = "testGSI", partition_key = "ref_id"))]
    #[allow(dead_code)]
    pub struct TypedGsiProjectionItem {
        ref_id: String,
        long_text: String,
    }

    #[derive(RaidenIndex, Debug, PartialEq)]
    #[raiden(source = "TypedCompositeGsiSortKeyTest", gsi = "testGSI")]
    #[raiden(gsi(
        name = "testGSI",
        partition_key = "ref_id",
        sort_key = "id",
        sort_key = "long_text"
    ))]
    #[allow(dead_code)]
    pub struct TypedCompositeGsiProjectionItem {
        ref_id: String,
        id: String,
        long_text: String,
    }

    #[derive(Raiden)]
    #[raiden(table_name = "LastEvaluateKeyData")]
    #[raiden(gsi(
        name = "userIndex",
        partition_key = "ref_id",
        sort_key = "id",
        sort_key = "long_text"
    ))]
    #[allow(dead_code)]
    pub struct User {
        #[raiden(partition_key)]
        id: String,
        ref_id: String,
        long_text: String,
        #[raiden(omit_gsi = "userIndex")]
        omitted: String,
    }

    #[test]
    fn test_gsi_partition_key_condition_builds() {
        let cond = TypedGsiPartitionKeyTest::test_gsi_key_condition().eq("id0");
        let (cond_str, attr_names, attr_values) = cond.build();

        assert!(cond_str.starts_with("#ref_id = :value"));
        assert_eq!(attr_names.get("#ref_id"), Some(&"ref_id".to_owned()));
        assert_eq!(attr_values.len(), 1);
    }

    #[test]
    fn test_gsi_projection_item_partition_key_condition_builds() {
        let cond = TypedGsiProjectionItem::ref_id().eq("id0");
        let (cond_str, attr_names, attr_values) = cond.build();

        assert!(cond_str.starts_with("#ref_id = :value"));
        assert_eq!(attr_names.get("#ref_id"), Some(&"ref_id".to_owned()));
        assert_eq!(attr_values.len(), 1);
    }

    #[tokio::test]
    async fn test_query_builder_accepts_gsi_partition_key_condition() {
        let client = crate::all::create_client_from_struct!(TypedGsiPartitionKeyTest);
        let cond = TypedGsiPartitionKeyTest::test_gsi_key_condition().eq("id0");
        let _builder = client.query().test_gsi().key_condition(cond);
    }

    #[tokio::test]
    async fn test_query_builder_accepts_projection_item_partition_key_condition() {
        let client = crate::all::create_client_from_struct!(TypedGsiProjectionSource);
        let cond = TypedGsiProjectionItem::ref_id().eq("id0");
        let _builder = client.query().test_gsi().key_condition(cond);
    }

    #[test]
    fn test_gsi_sort_key_condition_builds() {
        let cond = TypedGsiSortKeyTest::test_gsi_key_condition()
            .eq("id0")
            .and(TypedGsiSortKeyTest::test_gsi_sort_key_condition().begins_with("id"));
        let (cond_str, attr_names, attr_values) = cond.build();

        assert!(cond_str.starts_with("#ref_id = :value"));
        assert!(cond_str.contains("begins_with(#id, :value"));
        assert_eq!(attr_names.get("#ref_id"), Some(&"ref_id".to_owned()));
        assert_eq!(attr_names.get("#id"), Some(&"id".to_owned()));
        assert_eq!(attr_values.len(), 2);
    }

    #[tokio::test]
    async fn test_query_builder_accepts_gsi_sort_key_condition() {
        let client = crate::all::create_client_from_struct!(TypedGsiSortKeyTest);
        let cond = TypedGsiSortKeyTest::test_gsi_key_condition()
            .eq("id0")
            .and(TypedGsiSortKeyTest::test_gsi_sort_key_condition().begins_with("id"));
        let _builder = client.query().test_gsi().key_condition(cond);
    }

    #[test]
    fn test_composite_gsi_sort_key_conditions_build_in_order() {
        let cond = TypedCompositeGsiSortKeyTest::test_gsi_key_condition()
            .eq("id0")
            .and(TypedCompositeGsiSortKeyTest::test_gsi_sort_key_condition_1().eq("id1"))
            .and(TypedCompositeGsiSortKeyTest::test_gsi_sort_key_condition_2().begins_with("long"));
        let (cond_str, attr_names, attr_values) = cond.build();

        assert!(cond_str.starts_with("#ref_id = :value"));
        assert!(cond_str.contains("#id = :value"));
        assert!(cond_str.contains("begins_with(#long_text, :value"));
        assert_eq!(attr_names.get("#ref_id"), Some(&"ref_id".to_owned()));
        assert_eq!(attr_names.get("#id"), Some(&"id".to_owned()));
        assert_eq!(attr_names.get("#long_text"), Some(&"long_text".to_owned()));
        assert_eq!(attr_values.len(), 3);
    }

    #[test]
    fn test_composite_gsi_projection_item_sort_key_conditions_build_in_order() {
        let cond = TypedCompositeGsiProjectionItem::ref_id()
            .eq("id0")
            .and(TypedCompositeGsiProjectionItem::id().eq("id1"))
            .and(TypedCompositeGsiProjectionItem::long_text().begins_with("long"));
        let (cond_str, attr_names, attr_values) = cond.build();

        assert!(cond_str.starts_with("#ref_id = :value"));
        assert!(cond_str.contains("#id = :value"));
        assert!(cond_str.contains("begins_with(#long_text, :value"));
        assert_eq!(attr_names.get("#ref_id"), Some(&"ref_id".to_owned()));
        assert_eq!(attr_names.get("#id"), Some(&"id".to_owned()));
        assert_eq!(attr_names.get("#long_text"), Some(&"long_text".to_owned()));
        assert_eq!(attr_values.len(), 3);
    }

    #[test]
    fn test_auto_generated_gsi_projection_item_sort_key_conditions_build_in_order() {
        let cond = UserIndexItem::ref_id()
            .eq("id0")
            .and(UserIndexItem::id().eq("id1"))
            .and(UserIndexItem::long_text().begins_with("long"));
        let (cond_str, attr_names, attr_values) = cond.build();

        assert!(cond_str.starts_with("#ref_id = :value"));
        assert!(cond_str.contains("#id = :value"));
        assert!(cond_str.contains("begins_with(#long_text, :value"));
        assert_eq!(attr_names.get("#ref_id"), Some(&"ref_id".to_owned()));
        assert_eq!(attr_names.get("#id"), Some(&"id".to_owned()));
        assert_eq!(attr_names.get("#long_text"), Some(&"long_text".to_owned()));
        assert_eq!(attr_values.len(), 3);
    }

    #[tokio::test]
    async fn test_query_builder_accepts_composite_gsi_sort_key_conditions() {
        let client = crate::all::create_client_from_struct!(TypedCompositeGsiSortKeyTest);
        let cond = TypedCompositeGsiSortKeyTest::test_gsi_key_condition()
            .eq("id0")
            .and(TypedCompositeGsiSortKeyTest::test_gsi_sort_key_condition_1().eq("id1"))
            .and(TypedCompositeGsiSortKeyTest::test_gsi_sort_key_condition_2().begins_with("long"));
        let _builder = client.query().test_gsi().key_condition(cond);
    }

    #[tokio::test]
    async fn test_query_builder_accepts_composite_projection_item_gsi_sort_key_conditions() {
        let client = crate::all::create_client_from_struct!(TypedCompositeGsiSortKeyTest);
        let cond = TypedCompositeGsiProjectionItem::ref_id()
            .eq("id0")
            .and(TypedCompositeGsiProjectionItem::id().eq("id1"))
            .and(TypedCompositeGsiProjectionItem::long_text().begins_with("long"));
        let _builder = client.query().test_gsi().key_condition(cond);
    }

    #[tokio::test]
    async fn test_query_builder_keeps_deprecated_index_compatibility() {
        let client = crate::all::create_client_from_struct!(Test);
        let cond = Test::key_condition(Test::ref_id()).eq("id0");
        #[allow(deprecated)]
        let _builder = client.query().index("testGSI").key_condition(cond);
    }

    #[tokio::test]
    async fn test_typed_gsi_query_keeps_full_projection_without_projection_type() {
        let client = crate::all::create_client_from_struct!(TypedGsiProjectionSource);
        let builder = client.query().test_gsi();

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
    async fn test_typed_gsi_query_project_sets_projection_from_index_type() {
        let client = crate::all::create_client_from_struct!(TypedGsiProjectionSource);
        let builder = client
            .query()
            .test_gsi()
            .project::<TypedGsiProjectionItem>();

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
    async fn test_auto_generated_gsi_query_project_sets_projection_from_omit_gsi() {
        let client = crate::all::create_client_from_struct!(User);
        let builder = client.query().user_index().project::<UserIndexItem>();

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
            assert!(projection.contains("#id"));
            assert!(projection.contains("#long_text"));
            assert!(!projection.contains("#omitted"));
            assert!(names.contains_key("#ref_id"));
            assert!(names.contains_key("#id"));
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
            assert!(projection.contains("#id"));
            assert!(projection.contains("#long_text"));
            assert!(!projection.contains("#omitted"));
            assert!(names.contains_key("#ref_id"));
            assert!(names.contains_key("#id"));
            assert!(names.contains_key("#long_text"));
            assert!(!names.contains_key("#omitted"));
        }
    }

    #[tokio::test]
    async fn test_typed_gsi_query_run_with_accepts_index_type() {
        let client = crate::all::create_client_from_struct!(TypedGsiProjectionSource);
        let _future = client
            .query()
            .test_gsi()
            .run_with::<TypedGsiProjectionItem>();
    }

    #[tokio::test]
    async fn test_typed_gsi_query_project_can_chain_into_run_with() {
        fn assert_future_type<F>(_: F)
        where
            F: std::future::Future<
                Output = Result<query::QueryOutput<TypedGsiProjectionItem>, RaidenError>,
            >,
        {
        }

        let client = crate::all::create_client_from_struct!(TypedGsiProjectionSource);
        assert_future_type(
            client
                .query()
                .test_gsi()
                .project::<TypedGsiProjectionItem>()
                .run_with::<TypedGsiProjectionItem>(),
        );
    }

    #[tokio::test]
    async fn test_typed_gsi_query_project_run_returns_projection_output_type() {
        fn assert_future_type<F>(_: F)
        where
            F: std::future::Future<
                Output = Result<query::QueryOutput<TypedGsiProjectionItem>, RaidenError>,
            >,
        {
        }

        let client = crate::all::create_client_from_struct!(TypedGsiProjectionSource);
        assert_future_type(
            client
                .query()
                .test_gsi()
                .project::<TypedGsiProjectionItem>()
                .run(),
        );
    }

    #[tokio::test]
    async fn test_typed_gsi_query_project_preserves_output_type_after_key_condition() {
        fn assert_future_type<F>(_: F)
        where
            F: std::future::Future<
                Output = Result<query::QueryOutput<TypedGsiProjectionItem>, RaidenError>,
            >,
        {
        }

        let client = crate::all::create_client_from_struct!(TypedGsiProjectionSource);
        let cond = TypedGsiProjectionSource::test_gsi_key_condition().eq("id0");
        assert_future_type(
            client
                .query()
                .test_gsi()
                .project::<TypedGsiProjectionItem>()
                .key_condition(cond)
                .run(),
        );
    }

    #[tokio::test]
    async fn test_projection_item_query_starts_typed_query_builder() {
        fn assert_future_type<F>(_: F)
        where
            F: std::future::Future<
                Output = Result<query::QueryOutput<TypedGsiProjectionItem>, RaidenError>,
            >,
        {
        }

        let client = crate::all::create_client_from_struct!(TypedGsiProjectionSource);
        let cond = TypedGsiProjectionItem::ref_id().eq("id0");
        assert_future_type(
            TypedGsiProjectionItem::query(&client)
                .key_condition(cond)
                .run(),
        );
    }

    #[tokio::test]
    async fn test_auto_generated_projection_item_query_starts_typed_query_builder() {
        fn assert_future_type<F>(_: F)
        where
            F: std::future::Future<Output = Result<query::QueryOutput<UserIndexItem>, RaidenError>>,
        {
        }

        let client = crate::all::create_client_from_struct!(User);
        let cond = UserIndexItem::ref_id()
            .eq("id0")
            .and(UserIndexItem::id().eq("id1"))
            .and(UserIndexItem::long_text().begins_with("long"));
        assert_future_type(UserIndexItem::query(&client).key_condition(cond).run());
    }

    #[tokio::test]
    #[ignore = "requires a DynamoDB-compatible endpoint on localhost:8000"]
    async fn test_projection_item_query_decodes_projection_items() {
        let client = crate::all::create_client_from_struct!(TypedGsiProjectionSource);
        let cond = TypedGsiProjectionItem::ref_id().eq("id0");
        let res = TypedGsiProjectionItem::query(&client)
            .key_condition(cond)
            .run()
            .await
            .unwrap();

        assert_eq!(res.items.len(), 10);
        assert!(res
            .items
            .iter()
            .all(|item| item.ref_id == "id0" && !item.long_text.is_empty()));
    }

    #[tokio::test]
    async fn test_composite_projection_item_query_starts_typed_query_builder() {
        fn assert_future_type<F>(_: F)
        where
            F: std::future::Future<
                Output = Result<query::QueryOutput<TypedCompositeGsiProjectionItem>, RaidenError>,
            >,
        {
        }

        let client = crate::all::create_client_from_struct!(TypedCompositeGsiSortKeyTest);
        let cond = TypedCompositeGsiProjectionItem::ref_id()
            .eq("id0")
            .and(TypedCompositeGsiProjectionItem::id().eq("id1"))
            .and(TypedCompositeGsiProjectionItem::long_text().begins_with("long"));
        assert_future_type(
            TypedCompositeGsiProjectionItem::query(&client)
                .key_condition(cond)
                .run(),
        );
    }

    #[tokio::test]
    async fn test_deprecated_index_query_keeps_full_projection() {
        let client = crate::all::create_client_from_struct!(TypedGsiProjectionSource);
        #[allow(deprecated)]
        let builder = client.query().index("testGSI");

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

    #[tokio::test]
    async fn test_query_limit_1() {
        let client = crate::all::create_client_from_struct!(Test);
        let cond = Test::key_condition(Test::ref_id()).eq("id0");
        let res = client
            .query()
            .test_gsi()
            .limit(1)
            .key_condition(cond)
            .run()
            .await;
        assert_eq!(res.unwrap().items.len(), 1);
    }

    #[tokio::test]
    async fn test_query_limit_5() {
        let client = crate::all::create_client_from_struct!(Test);
        let cond = Test::key_condition(Test::ref_id()).eq("id0");
        let res = client
            .query()
            .test_gsi()
            .limit(5)
            .key_condition(cond)
            .run()
            .await;
        assert_eq!(res.unwrap().items.len(), 5);
    }

    #[tokio::test]
    async fn test_query_no_limit() {
        let client = crate::all::create_client_from_struct!(Test);
        let cond = Test::key_condition(Test::ref_id()).eq("id0");
        let res = client.query().test_gsi().key_condition(cond).run().await;
        assert_eq!(res.unwrap().items.len(), 10);
    }

    #[tokio::test]
    async fn test_query_over_limit() {
        let client = crate::all::create_client_from_struct!(Test);
        let cond = Test::key_condition(Test::ref_id()).eq("id0");
        let res = client
            .query()
            .test_gsi()
            .limit(11)
            .key_condition(cond)
            .run()
            .await;
        assert_eq!(res.unwrap().items.len(), 10);
    }

    #[tokio::test]
    async fn test_query_over_limit_with_next_token() {
        let client = crate::all::create_client_from_struct!(Test);
        let cond = Test::key_condition(Test::ref_id()).eq("id0");
        let res = client
            .query()
            .test_gsi()
            .limit(9)
            .key_condition(cond)
            .run()
            .await
            .unwrap();
        assert_eq!(res.items.len(), 9);
        assert!(res.next_token.is_some());
        let cond = Test::key_condition(Test::ref_id()).eq("id0");
        let res = client
            .query()
            .test_gsi()
            .limit(10)
            .next_token(res.next_token.unwrap())
            .key_condition(cond)
            .run()
            .await
            .unwrap();
        assert_eq!(res.items.len(), 1);
    }

    #[derive(Raiden)]
    #[raiden(table_name = "Project")]
    #[raiden(rename_all = "camelCase")]
    #[raiden(gsi = "orgIndex")]
    #[allow(dead_code)]
    pub struct Project {
        #[raiden(partition_key)]
        id: String,
        org_id: String,
        updated_at: String,
    }

    #[tokio::test]
    async fn test_query_with_renamed() {
        let client = crate::all::create_client_from_struct!(Project);
        let cond = Project::key_condition(Project::org_id()).eq("myOrg");
        let res = client
            .query()
            .org_index()
            .limit(11)
            .key_condition(cond)
            .run()
            .await;
        assert_eq!(res.unwrap().items.len(), 10);
    }

    #[derive(Raiden, Debug, PartialEq)]
    #[raiden(table_name = "QueryTestData0")]
    pub struct QueryTestData0a {
        #[raiden(partition_key)]
        id: String,
        name: String,
        year: usize,
    }

    #[tokio::test]
    async fn test_query_for_projection_expression() {
        let client = crate::all::create_client_from_struct!(QueryTestData0a);
        let cond = QueryTestData0a::key_condition(QueryTestData0a::id()).eq("id0");
        let res = client.query().key_condition(cond).run().await;

        assert_eq!(
            res.unwrap(),
            query::QueryOutput {
                consumed_capacity: None,
                count: Some(2),
                items: vec![
                    QueryTestData0a {
                        id: "id0".to_owned(),
                        name: "john".to_owned(),
                        year: 1999,
                    },
                    QueryTestData0a {
                        id: "id0".to_owned(),
                        name: "john".to_owned(),
                        year: 2000,
                    },
                ],
                next_token: None,
                scanned_count: Some(2),
            }
        );
    }

    #[derive(Raiden, Debug, PartialEq)]
    #[raiden(table_name = "QueryTestData1")]
    pub struct QueryTestData1 {
        #[raiden(partition_key)]
        id: String,
        #[raiden(sort_key)]
        name: String,
    }

    #[tokio::test]
    async fn test_query_with_begins_with_key_condition() {
        let client = crate::all::create_client_from_struct!(QueryTestData1);
        let cond = QueryTestData1::key_condition(QueryTestData1::id())
            .eq("id0")
            .and(QueryTestData1::key_condition(QueryTestData1::name()).begins_with("j"));
        let res = client.query().key_condition(cond).run().await;

        assert_eq!(
            res.unwrap(),
            query::QueryOutput {
                consumed_capacity: None,
                count: Some(2),
                items: vec![
                    QueryTestData1 {
                        id: "id0".to_owned(),
                        name: "jack".to_owned(),
                    },
                    QueryTestData1 {
                        id: "id0".to_owned(),
                        name: "john".to_owned(),
                    }
                ],
                next_token: None,
                scanned_count: Some(2),
            }
        );
    }

    #[derive(Raiden, Debug, PartialEq)]
    #[raiden(table_name = "QueryLargeDataTest")]
    #[raiden(gsi = "testGSI")]
    pub struct QueryLargeDataTest {
        #[raiden(partition_key)]
        id: String,
        ref_id: String,
        name: String,
    }

    #[tokio::test]
    async fn should_be_obtainable_when_the_size_is_1mb_or_larger() {
        let client = crate::all::create_client_from_struct!(QueryLargeDataTest);
        let cond = QueryLargeDataTest::key_condition(QueryLargeDataTest::ref_id()).eq("ref");
        let res = client.query().test_gsi().key_condition(cond).run().await;

        assert_eq!(res.unwrap().items.len(), 100);
    }

    #[tokio::test]
    async fn should_be_obtainable_specified_limit_items_when_the_size_is_1mb_or_larger() {
        let client = crate::all::create_client_from_struct!(QueryLargeDataTest);
        let cond = QueryLargeDataTest::key_condition(QueryLargeDataTest::ref_id()).eq("ref");
        let res = client
            .query()
            .test_gsi()
            .key_condition(cond)
            .limit(40)
            .run()
            .await
            .unwrap();

        assert_eq!(res.items.len(), 40);

        let token = res.next_token;

        let cond = QueryLargeDataTest::key_condition(QueryLargeDataTest::ref_id()).eq("ref");
        let res = client
            .query()
            .test_gsi()
            .key_condition(cond)
            .next_token(token.unwrap())
            .run()
            .await
            .unwrap();

        assert_eq!(res.items.len(), 60);
    }
}
