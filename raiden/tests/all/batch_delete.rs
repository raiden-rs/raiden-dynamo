#[cfg(test)]
mod partition_key_tests {

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

    #[test]
    fn test_batch_delete_item_for_stored_and_unstored_keys() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = BatchDeleteTest0::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });

            let res = client.batch_delete(vec!["id3", "unstored"]).run().await;
            assert!(res.is_ok());
        }
        rt.block_on(example());
    }

    #[test]
    fn test_batch_delete_item_for_unstored_keys() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = BatchDeleteTest0::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });

            let res = client
                .batch_delete(vec!["unstored0", "unstored1", "unstored2"])
                .run()
                .await;
            assert!(res.is_ok());
        }
        rt.block_on(example());
    }
}

#[cfg(test)]
mod partition_key_and_sort_key_tests {
    #[cfg(test)]
    use raiden::*;

    #[derive(Raiden, Debug, Clone, PartialEq)]
    pub struct BatchDeleteTest1 {
        #[raiden(partition_key)]
        id: String,
        name: String,
        #[raiden(sort_key)]
        year: usize,
    }

    #[test]
    fn test_batch_delete_item_with_sort_key() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = BatchDeleteTest1::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });

            let res = client
                .batch_delete(vec![
                    ("id0", 1999_usize),
                    ("id1", 2000_usize),
                    ("id2", 2001_usize),
                ])
                .run()
                .await;
            assert!(res.is_ok());
        }
        rt.block_on(example());
    }

    #[test]
    fn test_batch_delete_item_with_sort_key_for_stored_and_unstored_keys() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = BatchDeleteTest1::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });

            let res = client
                .batch_delete(vec![("id3", 2002_usize), ("unstored", 2000_usize)])
                .run()
                .await;
            assert!(res.is_ok());
        }
        rt.block_on(example());
    }

    #[test]
    fn test_batch_delete_item_with_sort_key_for_unstored_keys() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = BatchDeleteTest1::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });

            let res = client
                .batch_delete(vec![
                    ("unstored0", 1999_usize),
                    ("unstore1", 2000_usize),
                    ("unstored2", 2001_usize),
                ])
                .run()
                .await;
            assert!(res.is_ok());
        }
        rt.block_on(example());
    }
}
