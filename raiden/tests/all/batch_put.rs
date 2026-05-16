#[cfg(test)]
mod partition_key_tests {
    use pretty_assertions::assert_eq;
    use raiden::*;

    #[derive(Raiden, Debug, Clone, PartialEq)]
    pub struct BatchPutTest0 {
        #[raiden(partition_key)]
        id: String,
        name: String,
    }

    #[tokio::test]
    async fn test_batch_put_item() {
        let client = crate::all::create_client_from_struct!(BatchPutTest0);
        let items = (0..3)
            .map(|i| {
                BatchPutTest0::put_item_builder()
                    .id(format!("put-id{i}"))
                    .name("bob".to_owned())
                    .build()
            })
            .collect();

        let res = client.batch_put(items).run().await.unwrap();

        assert_eq!(
            res,
            batch_put::BatchPutOutput {
                consumed_capacity: None,
                unprocessed_items: vec![],
            }
        );

        let got = client.get("put-id0").run().await.unwrap();
        assert_eq!(
            got.item,
            BatchPutTest0 {
                id: "put-id0".to_owned(),
                name: "bob".to_owned(),
            }
        );
    }

    #[tokio::test]
    async fn test_batch_put_over_25_items() {
        let client = crate::all::create_client_from_struct!(BatchPutTest0);
        let items = (0..40)
            .map(|i| {
                BatchPutTest0::put_item_builder()
                    .id(format!("put-over-id{i}"))
                    .name("bob".to_owned())
                    .build()
            })
            .collect();

        let res = client.batch_put(items).run().await;

        assert!(res.is_ok());

        let first = client.get("put-over-id0").run().await.unwrap();
        let last = client.get("put-over-id39").run().await.unwrap();

        assert_eq!(
            first.item,
            BatchPutTest0 {
                id: "put-over-id0".to_owned(),
                name: "bob".to_owned(),
            }
        );
        assert_eq!(
            last.item,
            BatchPutTest0 {
                id: "put-over-id39".to_owned(),
                name: "bob".to_owned(),
            }
        );
    }

    #[tokio::test]
    async fn test_batch_put_overwrites_existing_items() {
        let client = crate::all::create_client_from_struct!(BatchPutTest0);
        let initial = BatchPutTest0::put_item_builder()
            .id("put-overwrite-id".to_owned())
            .name("before".to_owned())
            .build();
        let updated = BatchPutTest0::put_item_builder()
            .id("put-overwrite-id".to_owned())
            .name("after".to_owned())
            .build();

        client.batch_put(vec![initial]).run().await.unwrap();
        client.batch_put(vec![updated]).run().await.unwrap();

        let got = client.get("put-overwrite-id").run().await.unwrap();
        assert_eq!(
            got.item,
            BatchPutTest0 {
                id: "put-overwrite-id".to_owned(),
                name: "after".to_owned(),
            }
        );
    }
}

#[cfg(test)]
mod partition_key_and_sort_key_tests {
    use raiden::*;

    #[derive(Raiden, Debug, Clone, PartialEq)]
    pub struct BatchPutTest1 {
        #[raiden(partition_key)]
        id: String,
        name: String,
        #[raiden(sort_key)]
        year: usize,
    }

    #[tokio::test]
    async fn test_batch_put_item_with_sort_key() {
        let client = crate::all::create_client_from_struct!(BatchPutTest1);
        let items = (0..3)
            .map(|i| {
                BatchPutTest1::put_item_builder()
                    .id(format!("put-sk-id{i}"))
                    .name("bob".to_owned())
                    .year(2000 + i)
                    .build()
            })
            .collect();

        assert!(client.batch_put(items).run().await.is_ok());

        let got = client.get("put-sk-id0", 2000_usize).run().await.unwrap();
        assert_eq!(
            got.item,
            BatchPutTest1 {
                id: "put-sk-id0".to_owned(),
                name: "bob".to_owned(),
                year: 2000,
            }
        );
    }

    #[tokio::test]
    async fn test_batch_put_with_sort_key_over_25_items() {
        let client = crate::all::create_client_from_struct!(BatchPutTest1);
        let items = (0..40)
            .map(|i| {
                BatchPutTest1::put_item_builder()
                    .id(format!("put-sk-over-id{i}"))
                    .name("bob".to_owned())
                    .year(3000 + i)
                    .build()
            })
            .collect();

        client.batch_put(items).run().await.unwrap();

        let first = client
            .get("put-sk-over-id0", 3000_usize)
            .run()
            .await
            .unwrap();
        let last = client
            .get("put-sk-over-id39", 3039_usize)
            .run()
            .await
            .unwrap();

        assert_eq!(
            first.item,
            BatchPutTest1 {
                id: "put-sk-over-id0".to_owned(),
                name: "bob".to_owned(),
                year: 3000,
            }
        );
        assert_eq!(
            last.item,
            BatchPutTest1 {
                id: "put-sk-over-id39".to_owned(),
                name: "bob".to_owned(),
                year: 3039,
            }
        );
    }
}
