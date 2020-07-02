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
    }

    #[test]
    fn test_query() {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = QueryTestData0::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let cond = QueryTestData0::key_condition(QueryTestData0AttrNames::Id).eq("id0");
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
                        },
                        QueryTestData0 {
                            id: "id0".to_owned(),
                            name: "john".to_owned(),
                            year: 2000,
                            num: 2000,
                        },
                    ],
                    last_evaluated_key: None,
                    scanned_count: Some(2),
                }
            )
        }
        rt.block_on(example());
    }

    #[test]
    fn test_query_with_and_key_condition() {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = QueryTestData0::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let cond = QueryTestData0::key_condition(QueryTestData0AttrNames::Id)
                .eq("id0")
                .and(QueryTestData0::key_condition(QueryTestData0AttrNames::Year).eq(1999));
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
                    },],
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
    fn test_query_limit_1() {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = Test::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let cond = Test::key_condition(TestAttrNames::RefId).eq("id0");
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
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = Test::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let cond = Test::key_condition(TestAttrNames::RefId).eq("id0");
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
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = Test::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let cond = Test::key_condition(TestAttrNames::RefId).eq("id0");
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
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = Test::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let cond = Test::key_condition(TestAttrNames::RefId).eq("id0");
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
    fn test_query_with_renamed() {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = Project::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let cond = Project::key_condition(ProjectAttrNames::OrgId).eq("myOrg");
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
}
