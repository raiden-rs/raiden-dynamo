#[cfg(test)]
mod tests {

    #[cfg(test)]
    use pretty_assertions::assert_eq;
    use raiden::*;

    #[derive(Raiden, Debug, Clone, PartialEq)]
    pub struct BatchTest0 {
        #[raiden(partition_key)]
        id: String,
        name: String,
    }

    fn sort_by_id_0(
        mut output: batch_get::BatchGetOutput<BatchTest0>,
    ) -> batch_get::BatchGetOutput<BatchTest0> {
        output.items.sort_by_key(|i| {
            let mut id = i.id.to_string();
            id.replace_range(0..2, "");
            id.parse::<i32>().unwrap()
        });
        batch_get::BatchGetOutput { ..output }
    }

    #[test]
    fn test_batch_get_item() {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = BatchTest0::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });

            let res: batch_get::BatchGetOutput<BatchTest0> = client
                .batch_get(vec!["id0", "id1", "id2"])
                .run()
                .await
                .unwrap();
            assert_eq!(
                sort_by_id_0(res),
                batch_get::BatchGetOutput {
                    items: vec![
                        BatchTest0 {
                            id: "id0".to_owned(),
                            name: "bob".to_owned(),
                        },
                        BatchTest0 {
                            id: "id1".to_owned(),
                            name: "bob".to_owned(),
                        },
                        BatchTest0 {
                            id: "id2".to_owned(),
                            name: "bob".to_owned(),
                        },
                    ],
                    consumed_capacity: None,
                    unprocessed_keys: Some(KeysAndAttributes {
                        attributes_to_get: None,
                        consistent_read: None,
                        expression_attribute_names: None,
                        keys: vec![],
                        projection_expression: None,
                    }),
                }
            );
        }
        rt.block_on(example());
    }

    #[test]
    fn test_batch_get_item_extended() {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = BatchTest0::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });

            let keys: Vec<String> = (0..101).into_iter().map(|n| format!("id{}", n)).collect();
            let expected_items = (0..101)
                .map(|n| BatchTest0 {
                    id: format!("id{}", n),
                    name: "bob".to_owned(),
                })
                .collect();
            let res: batch_get::BatchGetOutput<BatchTest0> =
                client.batch_get(keys).run().await.unwrap();
            assert_eq!(
                sort_by_id_0(res),
                batch_get::BatchGetOutput {
                    items: expected_items,
                    consumed_capacity: None,
                    unprocessed_keys: Some(KeysAndAttributes {
                        attributes_to_get: None,
                        consistent_read: None,
                        expression_attribute_names: None,
                        keys: vec![],
                        projection_expression: None,
                    }),
                }
            );
        }
        rt.block_on(example());
    }

    // NOTE: Same behavior with original SDK, but we're planning to improve this.
    // ref. https://github.com/raiden-rs/raiden/issues/44
    #[test]
    fn test_batch_get_item_partial_missing() {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = BatchTest0::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });

            let res: batch_get::BatchGetOutput<BatchTest0> = client
                .batch_get(vec!["id100", "id101", "id102"])
                .run()
                .await
                .unwrap();
            assert_eq!(
                sort_by_id_0(res),
                batch_get::BatchGetOutput {
                    items: vec![BatchTest0 {
                        id: "id100".to_owned(),
                        name: "bob".to_owned(),
                    },],
                    consumed_capacity: None,
                    unprocessed_keys: Some(KeysAndAttributes {
                        attributes_to_get: None,
                        consistent_read: None,
                        expression_attribute_names: None,
                        keys: vec![],
                        projection_expression: None,
                    }),
                }
            );
        }
        rt.block_on(example());
    }

    #[derive(Raiden, Debug, PartialEq)]
    pub struct BatchTest1 {
        #[raiden(partition_key)]
        id: String,
        name: String,
        #[raiden(sort_key)]
        year: usize,
        num: usize,
    }

    fn sort_by_id_1(
        mut output: batch_get::BatchGetOutput<BatchTest1>,
    ) -> batch_get::BatchGetOutput<BatchTest1> {
        output.items.sort_by_key(|i| {
            let mut id = i.id.to_string();
            id.replace_range(0..2, "");
            id.parse::<i32>().unwrap()
        });
        batch_get::BatchGetOutput { ..output }
    }

    #[test]
    fn test_batch_get_item_sort_key() {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = BatchTest1::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });

            let keys: Vec<(String, i32)> = (0..250)
                .into_iter()
                .map(|n| (format!("id{}", n), 2000 + n))
                .collect();
            let expected_items = (0..250)
                .map(|n| BatchTest1 {
                    id: format!("id{}", n),
                    name: "bob".to_owned(),
                    year: 2000 + n,
                    num: n,
                })
                .collect();
            let res: batch_get::BatchGetOutput<BatchTest1> =
                client.batch_get(keys).run().await.unwrap();
            assert_eq!(
                sort_by_id_1(res),
                batch_get::BatchGetOutput {
                    items: expected_items,
                    consumed_capacity: None,
                    unprocessed_keys: Some(KeysAndAttributes {
                        attributes_to_get: None,
                        consistent_read: None,
                        expression_attribute_names: None,
                        keys: vec![],
                        projection_expression: None,
                    }),
                }
            );
        }
        rt.block_on(example());
    }

    #[derive(Raiden, Debug, PartialEq)]
    #[raiden(table_name = "BatchTest1")]
    pub struct BatchTest1a {
        #[raiden(partition_key)]
        id: String,
        name: String,
        #[raiden(sort_key)]
        year: usize,
    }

    #[test]
    fn test_batch_get_item_for_projection_expression() {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = BatchTest1a::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });

            let keys: Vec<(String, i32)> = (0..250)
                .into_iter()
                .map(|n| (format!("id{}", n), 2000 + n))
                .collect();
            let expected_items = (0..250)
                .map(|n| BatchTest1a {
                    id: format!("id{}", n),
                    name: "bob".to_owned(),
                    year: 2000 + n,
                })
                .collect();

            let mut res: batch_get::BatchGetOutput<BatchTest1a> =
                client.batch_get(keys).run().await.unwrap();
            res.items.sort_by_key(|i| {
                let mut id = i.id.to_string();
                id.replace_range(0..2, "");
                id.parse::<i32>().unwrap()
            });

            assert_eq!(
                res,
                batch_get::BatchGetOutput {
                    items: expected_items,
                    consumed_capacity: None,
                    unprocessed_keys: Some(KeysAndAttributes {
                        attributes_to_get: None,
                        consistent_read: None,
                        expression_attribute_names: None,
                        keys: vec![],
                        projection_expression: None,
                    }),
                }
            );
        }
        rt.block_on(example());
    }

    #[test]
    fn test_batch_get_item_missing_all() {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = BatchTest1::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });

            let res: batch_get::BatchGetOutput<BatchTest1> = client
                .batch_get(vec![("id300", 2300), ("id301", 2301), ("id302", 2302)])
                .run()
                .await
                .unwrap();
            assert_eq!(
                sort_by_id_1(res),
                batch_get::BatchGetOutput {
                    items: vec![],
                    consumed_capacity: None,
                    unprocessed_keys: Some(KeysAndAttributes {
                        attributes_to_get: None,
                        consistent_read: None,
                        expression_attribute_names: None,
                        keys: vec![],
                        projection_expression: None,
                    }),
                }
            );
        }
        rt.block_on(example());
    }

    #[derive(Raiden, Debug, Clone, PartialEq)]
    pub struct BatchTest2 {
        #[raiden(partition_key)]
        id: String,
        name: String,
    }
    #[test]
    fn test_batch_get_item_with_16mb_limitation() {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = BatchTest2::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });

            let res: batch_get::BatchGetOutput<BatchTest2> = client
                .batch_get(vec![
                    "id0", "id1", "id2", "id3", "id4", "id5", "id6", "id7", "id8", "id9", "id10",
                    "id11", "id12", "id13", "id14", "id15", "id16", "id17", "id18", "id19", "id20",
                    "id21", "id22", "id23", "id24", "id25", "id26", "id27", "id28", "id29", "id30",
                    "id31", "id32", "id33", "id34", "id35", "id36", "id37", "id38", "id39", "id40",
                    "id41", "id42", "id43", "id44", "id45", "id46", "id47", "id48", "id49", "id50",
                ])
                .run()
                .await
                .unwrap();
            assert_eq!(res.items.len(), 51);
        }
        rt.block_on(example());
    }
}
