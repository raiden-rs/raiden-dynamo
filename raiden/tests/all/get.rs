#[cfg(test)]
mod tests {

    #[cfg(test)]
    use pretty_assertions::assert_eq;
    use raiden::*;

    #[derive(Raiden)]
    #[raiden(table_name = "user")]
    #[derive(Debug, Clone, PartialEq)]
    pub struct User {
        #[raiden(partition_key)]
        id: String,
        name: String,
        num_usize: usize,
        num_u8: u8,
        num_i8: i8,
        option_u16: Option<u16>,
        option_i16: Option<i16>,
    }

    #[test]
    fn test_user_get_item() {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = User::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });

            let res = client.get("user_primary_key").run().await;
            assert_eq!(
                res.unwrap(),
                get::GetOutput {
                    item: User {
                        id: "user_primary_key".to_owned(),
                        name: "bokuweb".to_owned(),
                        num_usize: 42,
                        num_u8: 255,
                        num_i8: -127,
                        option_u16: None,
                        option_i16: Some(-1),
                    },
                    consumed_capacity: None,
                }
            );
        }
        rt.block_on(example());
    }

    #[test]
    fn test_user_get_item_with_consistent_read() {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = UserClient::new(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let res = client.get("user_primary_key").consistent().run().await;
            assert_eq!(
                res.unwrap(),
                get::GetOutput {
                    item: User {
                        id: "user_primary_key".to_owned(),
                        name: "bokuweb".to_owned(),
                        num_usize: 42,
                        num_u8: 255,
                        num_i8: -127,
                        option_u16: None,
                        option_i16: Some(-1),
                    },
                    consumed_capacity: None,
                }
            );
        }
        rt.block_on(example());
    }

    #[test]
    fn test_user_get_item_with_not_found_error() {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = UserClient::new(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let res = client.get("not_exist_key").consistent().run().await;
            assert_eq!(
                res,
                Err(RaidenError::ResourceNotFound(
                    "resource not found".to_owned()
                )),
            );
        }
        rt.block_on(example());
    }

    #[derive(Raiden)]
    #[raiden(table_name = "user")]
    #[derive(Debug, Clone, PartialEq)]
    pub struct UserWithUnStored {
        #[raiden(partition_key)]
        id: String,
        name: String,
        unstored: usize,
    }

    #[test]
    fn test_get_unstored_value() {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = UserWithUnStored::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let res = client.get("user_primary_key").consistent().run().await;
            assert_eq!(
                res,
                // Err(RaidenError::AttributeValueNotFoundError {
                //     attr_name: "unstored".to_owned(),
                // }),
                Err(RaidenError::AttributeConvertError {
                    attr_name: "unstored".to_owned(),
                }),
            );
        }
        rt.block_on(example());
    }

    #[derive(Raiden)]
    #[raiden(table_name = "user")]
    #[derive(Debug, Clone, PartialEq)]
    pub struct UserWithEmptyHashSet {
        #[raiden(partition_key)]
        id: String,
        name: String,
        empty_set: std::collections::HashSet<usize>,
    }

    #[test]
    fn test_get_empty_hashset() {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = UserWithEmptyHashSet::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let res = client.get("user_primary_key").consistent().run().await;
            assert_eq!(
                res.unwrap(),
                get::GetOutput {
                    item: UserWithEmptyHashSet {
                        id: "user_primary_key".to_owned(),
                        name: "bokuweb".to_owned(),
                        empty_set: std::collections::HashSet::new(),
                    },
                    consumed_capacity: None,
                }
            );
        }
        rt.block_on(example());
    }

    #[derive(Raiden)]
    #[raiden(table_name = "user")]
    #[derive(Debug, Clone, PartialEq)]
    pub struct UserWithEmptyVec {
        #[raiden(partition_key)]
        id: String,
        name: String,
        empty_vec: Vec<usize>,
    }

    #[test]
    fn test_get_empty_vec() {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = UserWithEmptyVec::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let res = client.get("user_primary_key").consistent().run().await;
            assert_eq!(
                res.unwrap(),
                get::GetOutput {
                    item: UserWithEmptyVec {
                        id: "user_primary_key".to_owned(),
                        name: "bokuweb".to_owned(),
                        empty_vec: vec![],
                    },
                    consumed_capacity: None,
                }
            );
        }
        rt.block_on(example());
    }
}
