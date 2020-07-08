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

    fn sort(
        mut output: batch_get::BatchGetOutput<BatchTest0>,
    ) -> batch_get::BatchGetOutput<BatchTest0> {
        output.items.sort_by_key(|i| {
            let mut id = i.id.to_string();
            id.replace_range(0..2, "");
            id.parse::<usize>().unwrap()
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

            let res = client.batch_get(vec!["id0", "id1", "id2"]).run().await;
            let res: batch_get::BatchGetOutput<BatchTest0> = res.unwrap();
            assert_eq!(
                sort(res),
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

    #[derive(Raiden, Debug, PartialEq)]
    pub struct BatchTest1 {
        #[raiden(partition_key)]
        id: String,
        name: String,
        #[raiden(sort_key)]
        year: usize,
        num: usize,
    }
}
