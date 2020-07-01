#[cfg(test)]
mod tests {

    #[cfg(test)]
    use pretty_assertions::assert_eq;
    use raiden::*;

    #[derive(Raiden)]
    #[raiden(table_name = "RenameAllCamelCaseTestData0")]
    #[raiden(rename_all = "camelCase")]
    #[derive(Debug, Clone, PartialEq)]
    pub struct RenameAllCamelCaseTest {
        #[raiden(partition_key)]
        partition_key: String,
        foo_bar: String,
        project_id: usize,
    }

    #[test]
    fn test_rename_all_camelcase_get() {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = RenameAllCamelCaseTest::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });

            let res = client.get("id0").run().await;
            assert_eq!(
                res.unwrap(),
                get::GetOutput {
                    item: RenameAllCamelCaseTest {
                        partition_key: "id0".to_owned(),
                        foo_bar: "john".to_owned(),
                        project_id: 1,
                    },
                    consumed_capacity: None,
                }
            );
        }
        rt.block_on(example());
    }

    #[derive(Raiden)]
    #[raiden(table_name = "RenameAllPascalCaseTestData0")]
    #[raiden(rename_all = "PascalCase")]
    #[derive(Debug, Clone, PartialEq)]
    pub struct RenameAllPascalCaseTest {
        #[raiden(partition_key)]
        partition_key: String,
        foo_bar: String,
        project_id: usize,
    }

    #[test]
    fn test_rename_all_pascalcase_get() {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = RenameAllPascalCaseTest::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });

            let res = client.get("id0").run().await;
            assert_eq!(
                res.unwrap(),
                get::GetOutput {
                    item: RenameAllPascalCaseTest {
                        partition_key: "id0".to_owned(),
                        foo_bar: "john".to_owned(),
                        project_id: 1,
                    },
                    consumed_capacity: None,
                }
            );
        }
        rt.block_on(example());
    }
}
