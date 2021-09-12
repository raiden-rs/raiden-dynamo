#[cfg(test)]
mod tests {

    #[cfg(test)]
    use pretty_assertions::assert_eq;
    use raiden::*;

    #[derive(Raiden, Debug, Clone, PartialEq)]
    pub struct BatchDeleteTest0 {
        #[raiden(partition_key)]
        id: String,
        name: String,
    }

    #[test]
    fn test_batch_delete_item() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = BatchDeleteTest0::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });

            let res: batch_delete::BatchDeleteOutput = client
                .batch_delete(vec!["id0", "id1", "id2"])
                .run()
                .await
                .unwrap();
            assert_eq!(
                res,
                batch_delete::BatchDeleteOutput {
                    consumed_capacity: None,
                    unprocessed_items: vec![],
                }
            );
        }
        rt.block_on(example());
    }
}
