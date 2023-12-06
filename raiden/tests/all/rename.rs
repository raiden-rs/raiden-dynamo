#[cfg(test)]
mod tests {

    #[cfg(test)]
    use pretty_assertions::assert_eq;
    use raiden::*;

    #[derive(Raiden)]
    #[raiden(table_name = "RenameTestData0")]
    #[derive(Debug, Clone, PartialEq)]
    pub struct RenameTest {
        #[raiden(partition_key)]
        id: String,
        name: String,
        #[raiden(rename = "renamed")]
        before_rename: usize,
    }

    #[derive(Raiden)]
    #[raiden(table_name = "RenameTestData0")]
    #[derive(Debug, Clone, PartialEq)]
    pub struct RenameKeyTest {
        #[raiden(partition_key)]
        #[raiden(rename = "id")]
        before_renamed_id: String,
        name: String,
        #[raiden(rename = "renamed")]
        before_rename: usize,
    }

    #[tokio::test]
    async fn test_rename_get_item() {
        let client = crate::all::create_client_from_struct!(RenameTest);
        let res = client.get("id0").run().await;

        assert_eq!(
            res.unwrap(),
            get::GetOutput {
                item: RenameTest {
                    id: "id0".to_owned(),
                    name: "john".to_owned(),
                    before_rename: 1999,
                },
                consumed_capacity: None,
            }
        );
        assert_eq!(
            RenameTestAttrNames::Renamed.into_attr_name(),
            "renamed".to_owned()
        );
    }

    #[tokio::test]
    async fn test_rename_key_get_item() {
        let client = crate::all::create_client_from_struct!(RenameKeyTest);
        let res = client.get("id0").run().await;

        assert_eq!(
            res.unwrap(),
            get::GetOutput {
                item: RenameKeyTest {
                    before_renamed_id: "id0".to_owned(),
                    name: "john".to_owned(),
                    before_rename: 1999,
                },
                consumed_capacity: None,
            }
        );
    }

    #[tokio::test]
    async fn test_rename_query() {
        let client = crate::all::create_client_from_struct!(RenameTest);
        let cond = RenameTest::key_condition(RenameTest::id()).eq("id0");
        let res = client.query().key_condition(cond).run().await;

        assert_eq!(
            res.unwrap(),
            query::QueryOutput {
                consumed_capacity: None,
                count: Some(1),
                items: vec![RenameTest {
                    id: "id0".to_owned(),
                    name: "john".to_owned(),
                    before_rename: 1999,
                },],
                next_token: None,
                scanned_count: Some(1),
            }
        );
    }
}
