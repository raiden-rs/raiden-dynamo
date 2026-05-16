#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use raiden::*;

    #[derive(Raiden, Debug, Clone, PartialEq)]
    pub struct BatchTest0 {
        #[raiden(partition_key)]
        id: String,
        name: String,
    }

    #[derive(Raiden, Debug, Clone, PartialEq)]
    pub struct BatchTest1 {
        #[raiden(partition_key)]
        id: String,
        name: String,
        #[raiden(sort_key)]
        year: usize,
    }

    #[tokio::test]
    async fn test_transact_get_items() {
        let client = crate::all::create_client_from_struct!(BatchTest0);
        let res = client
            .transact_get(vec!["id0", "id1", "unstored"])
            .run()
            .await
            .unwrap();

        assert_eq!(res.items.len(), 3);
        assert_eq!(
            res.items[0],
            Some(BatchTest0 {
                id: "id0".to_owned(),
                name: "bob".to_owned(),
            })
        );
        assert_eq!(res.items[2], None);
    }

    #[tokio::test]
    async fn test_transact_get_items_preserves_request_order() {
        let client = crate::all::create_client_from_struct!(BatchTest0);
        let res = client
            .transact_get(vec!["id2", "unstored", "id0", "id1"])
            .run()
            .await
            .unwrap();

        assert_eq!(
            res.items,
            vec![
                Some(BatchTest0 {
                    id: "id2".to_owned(),
                    name: "bob".to_owned(),
                }),
                None,
                Some(BatchTest0 {
                    id: "id0".to_owned(),
                    name: "bob".to_owned(),
                }),
                Some(BatchTest0 {
                    id: "id1".to_owned(),
                    name: "bob".to_owned(),
                }),
            ]
        );
    }

    #[tokio::test]
    async fn test_transact_get_items_with_sort_key() {
        let client = crate::all::create_client_from_struct!(BatchTest1);
        let res = client
            .transact_get(vec![("id0", 2000_usize), ("id1", 2001_usize)])
            .run()
            .await
            .unwrap();

        assert_eq!(res.items.len(), 2);
        assert_eq!(
            res.items[0],
            Some(BatchTest1 {
                id: "id0".to_owned(),
                name: "bob".to_owned(),
                year: 2000,
            })
        );
    }

    #[tokio::test]
    async fn test_transact_get_items_with_sort_key_and_missing_item() {
        let client = crate::all::create_client_from_struct!(BatchTest1);
        let res = client
            .transact_get(vec![
                ("id1", 2001_usize),
                ("id1", 2000_usize),
                ("id2", 2002_usize),
            ])
            .run()
            .await
            .unwrap();

        assert_eq!(
            res.items,
            vec![
                Some(BatchTest1 {
                    id: "id1".to_owned(),
                    name: "bob".to_owned(),
                    year: 2001,
                }),
                None,
                Some(BatchTest1 {
                    id: "id2".to_owned(),
                    name: "bob".to_owned(),
                    year: 2002,
                }),
            ]
        );
    }

    #[derive(Raiden, Debug, Clone, PartialEq)]
    #[raiden(table_name = "BatchTest1")]
    pub struct BatchTest1Projection {
        #[raiden(partition_key)]
        id: String,
        name: String,
        #[raiden(sort_key)]
        year: usize,
    }

    #[tokio::test]
    async fn test_transact_get_items_uses_projection_expression() {
        let client = crate::all::create_client_from_struct!(BatchTest1Projection);
        let res = client
            .transact_get(vec![("id0", 2000_usize)])
            .run()
            .await
            .unwrap();

        assert_eq!(
            res.items,
            vec![Some(BatchTest1Projection {
                id: "id0".to_owned(),
                name: "bob".to_owned(),
                year: 2000,
            })]
        );
    }
}
