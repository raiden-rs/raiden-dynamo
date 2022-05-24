#[cfg(test)]
mod tests {

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

    #[test]
    fn test_query() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = QueryTestData0::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
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
            )
        }
        rt.block_on(example());
    }

    #[test]
    fn test_query_with_and_key_condition() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = QueryTestData0::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
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
            )
        }
        rt.block_on(example());
    }

    #[tokio::test]
    async fn test_query_with_simple_filter() {
        let client = QueryTestData0::client(Region::Custom {
            endpoint: "http://localhost:8000".into(),
            name: "ap-northeast-1".into(),
        });
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
    async fn test_query_with_not_function_filter() {
        let client = QueryTestData0::client(Region::Custom {
            endpoint: "http://localhost:8000".into(),
            name: "ap-northeast-1".into(),
        });
        let cond = QueryTestData0::key_condition(QueryTestData0::id()).eq("id3");
        let filter = QueryTestData0::filter_expression_with_not(
            QueryTestData0::filter_expression(QueryTestData0::name()).eq("bar1"),
        );
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
        let client = QueryTestData0::client(Region::Custom {
            endpoint: "http://localhost:8000".into(),
            name: "ap-northeast-1".into(),
        });
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
        let client = QueryTestData0::client(Region::Custom {
            endpoint: "http://localhost:8000".into(),
            name: "ap-northeast-1".into(),
        });
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
        let client = QueryTestData0::client(Region::Custom {
            endpoint: "http://localhost:8000".into(),
            name: "ap-northeast-1".into(),
        });
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
        let client = QueryTestData0::client(Region::Custom {
            endpoint: "http://localhost:8000".into(),
            name: "ap-northeast-1".into(),
        });
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
        let client = QueryTestData0::client(Region::Custom {
            endpoint: "http://localhost:8000".into(),
            name: "ap-northeast-1".into(),
        });
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

    #[derive(Raiden)]
    #[raiden(table_name = "LastEvaluateKeyData")]
    #[allow(dead_code)]
    pub struct Test {
        #[raiden(partition_key)]
        id: String,
        ref_id: String,
        long_text: String,
    }

    #[test]
    fn test_query_limit_1() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = Test::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let cond = Test::key_condition(Test::ref_id()).eq("id0");
            let res = client
                .query()
                .index("testGSI")
                .limit(1)
                .key_condition(cond)
                .run()
                .await;
            assert_eq!(res.unwrap().items.len(), 1);
        }
        rt.block_on(example());
    }

    #[test]
    fn test_query_limit_5() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = Test::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let cond = Test::key_condition(Test::ref_id()).eq("id0");
            let res = client
                .query()
                .index("testGSI")
                .limit(5)
                .key_condition(cond)
                .run()
                .await;
            assert_eq!(res.unwrap().items.len(), 5);
        }
        rt.block_on(example());
    }

    #[test]
    fn test_query_no_limit() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = Test::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let cond = Test::key_condition(Test::ref_id()).eq("id0");
            let res = client
                .query()
                .index("testGSI")
                .key_condition(cond)
                .run()
                .await;
            assert_eq!(res.unwrap().items.len(), 10);
        }
        rt.block_on(example());
    }

    #[test]
    fn test_query_over_limit() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = Test::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let cond = Test::key_condition(Test::ref_id()).eq("id0");
            let res = client
                .query()
                .index("testGSI")
                .limit(11)
                .key_condition(cond)
                .run()
                .await;
            assert_eq!(res.unwrap().items.len(), 10);
        }
        rt.block_on(example());
    }

    #[test]
    fn test_query_over_limit_with_next_token() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = Test::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let cond = Test::key_condition(Test::ref_id()).eq("id0");
            let res = client
                .query()
                .index("testGSI")
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
                .index("testGSI")
                .limit(10)
                .next_token(res.next_token.unwrap())
                .key_condition(cond)
                .run()
                .await
                .unwrap();
            assert_eq!(res.items.len(), 1);
        }
        rt.block_on(example());
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

    #[test]
    fn test_query_with_renamed() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = Project::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let cond = Project::key_condition(Project::org_id()).eq("myOrg");
            let res = client
                .query()
                .index("orgIndex")
                .limit(11)
                .key_condition(cond)
                .run()
                .await;
            assert_eq!(res.unwrap().items.len(), 10);
        }
        rt.block_on(example());
    }

    #[derive(Raiden, Debug, PartialEq)]
    #[raiden(table_name = "QueryTestData0")]
    pub struct QueryTestData0a {
        #[raiden(partition_key)]
        id: String,
        name: String,
        year: usize,
    }
    #[test]
    fn test_query_for_projection_expression() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = QueryTestData0a::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
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
            )
        }
        rt.block_on(example());
    }

    #[derive(Raiden, Debug, PartialEq)]
    #[raiden(table_name = "QueryTestData1")]
    pub struct QueryTestData1 {
        #[raiden(partition_key)]
        id: String,
        #[raiden(sort_key)]
        name: String,
    }

    #[test]
    fn test_query_with_begins_with_key_condition() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = QueryTestData1::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
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
            )
        }
        rt.block_on(example());
    }

    #[derive(Raiden, Debug, PartialEq)]
    #[raiden(table_name = "QueryLargeDataTest")]
    pub struct QueryLargeDataTest {
        #[raiden(partition_key)]
        id: String,
        ref_id: String,
        name: String,
    }

    #[test]
    fn should_be_obtainable_when_the_size_is_1mb_or_larger() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = QueryLargeDataTest::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let cond = QueryLargeDataTest::key_condition(QueryLargeDataTest::ref_id()).eq("ref");
            let res = client
                .query()
                .index("testGSI")
                .key_condition(cond)
                .run()
                .await;

            assert_eq!(res.unwrap().items.len(), 100)
        }
        rt.block_on(example());
    }

    #[test]
    fn should_be_obtainable_specified_limit_items_when_the_size_is_1mb_or_larger() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = QueryLargeDataTest::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let cond = QueryLargeDataTest::key_condition(QueryLargeDataTest::ref_id()).eq("ref");
            let res = client
                .query()
                .index("testGSI")
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
                .index("testGSI")
                .key_condition(cond)
                .next_token(token.unwrap())
                .run()
                .await
                .unwrap();

            assert_eq!(res.items.len(), 60);
        }
        rt.block_on(example());
    }
}
