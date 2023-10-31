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
        async fn example() {
            let client = crate::all::create_client_from_struct!(BatchDeleteTest0);
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

        tokio::runtime::Runtime::new().unwrap().block_on(example());
    }

    #[test]
    fn test_batch_delete_item_for_stored_and_unstored_keys() {
        async fn example() {
            let client = crate::all::create_client_from_struct!(BatchDeleteTest0);
            let res = client.batch_delete(vec!["id3", "unstored"]).run().await;

            assert!(res.is_ok());
        }

        tokio::runtime::Runtime::new().unwrap().block_on(example());
    }

    #[test]
    fn test_batch_delete_item_for_unstored_keys() {
        async fn example() {
            let client = crate::all::create_client_from_struct!(BatchDeleteTest0);
            let res = client
                .batch_delete(vec!["unstored0", "unstored1", "unstored2"])
                .run()
                .await;

            assert!(res.is_ok());
        }

        tokio::runtime::Runtime::new().unwrap().block_on(example());
    }

    #[test]
    fn test_batch_delete_over_25_items() {
        async fn example() {
            let client = crate::all::create_client_from_struct!(BatchDeleteTest0);
            let res = client
                .batch_delete((4..=100).map(|i| format!("id{i}")).collect())
                .run()
                .await;

            assert!(res.is_ok());
        }

        tokio::runtime::Runtime::new().unwrap().block_on(example());
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
        async fn example() {
            let client = crate::all::create_client_from_struct!(BatchDeleteTest1);
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

        tokio::runtime::Runtime::new().unwrap().block_on(example());
    }

    #[test]
    fn test_batch_delete_item_with_sort_key_for_stored_and_unstored_keys() {
        async fn example() {
            let client = crate::all::create_client_from_struct!(BatchDeleteTest1);
            let res = client
                .batch_delete(vec![("id3", 2002_usize), ("unstored", 2000_usize)])
                .run()
                .await;

            assert!(res.is_ok());
        }

        tokio::runtime::Runtime::new().unwrap().block_on(example());
    }

    #[test]
    fn test_batch_delete_item_with_sort_key_for_unstored_keys() {
        async fn example() {
            let client = crate::all::create_client_from_struct!(BatchDeleteTest1);
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

        tokio::runtime::Runtime::new().unwrap().block_on(example());
    }

    #[test]
    fn test_batch_delete_with_sort_key_over_25_items() {
        async fn example() {
            let client = crate::all::create_client_from_struct!(BatchDeleteTest1);
            let res = client
                .batch_delete(
                    (4..=100)
                        .map(|i| (format!("id{i}"), 1999_usize + i))
                        .collect(),
                )
                .run()
                .await;

            assert!(res.is_ok());
        }

        tokio::runtime::Runtime::new().unwrap().block_on(example());
    }
}
