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
        let rt = tokio::runtime::Runtime::new().unwrap();
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
        let rt = tokio::runtime::Runtime::new().unwrap();
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
        let rt = tokio::runtime::Runtime::new().unwrap();
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
        let rt = tokio::runtime::Runtime::new().unwrap();
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
        let rt = tokio::runtime::Runtime::new().unwrap();
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
        let rt = tokio::runtime::Runtime::new().unwrap();
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

    #[derive(Raiden)]
    #[raiden(table_name = "user")]
    #[derive(Debug, Clone, PartialEq)]
    pub struct UserWithStringSet {
        #[raiden(partition_key)]
        id: String,
        name: String,
        string_set: std::collections::HashSet<String>,
    }

    #[test]
    fn test_get_stringset() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = UserWithStringSet::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let res = client.get("user_primary_key").consistent().run().await;
            let mut set = std::collections::HashSet::new();
            set.insert("Hello".to_owned());
            assert_eq!(
                res.unwrap(),
                get::GetOutput {
                    item: UserWithStringSet {
                        id: "user_primary_key".to_owned(),
                        name: "bokuweb".to_owned(),
                        string_set: set,
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
    pub struct UserWithStringBTreeSet {
        #[raiden(partition_key)]
        id: String,
        name: String,
        string_set: std::collections::BTreeSet<String>,
    }

    #[test]
    fn test_get_btree_stringset() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = UserWithStringBTreeSet::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let res = client.get("user_primary_key").consistent().run().await;
            let mut set = std::collections::BTreeSet::new();
            set.insert("Hello".to_owned());
            assert_eq!(
                res.unwrap(),
                get::GetOutput {
                    item: UserWithStringBTreeSet {
                        id: "user_primary_key".to_owned(),
                        name: "bokuweb".to_owned(),
                        string_set: set,
                    },
                    consumed_capacity: None,
                }
            );
        }
        rt.block_on(example());
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct CustomSSItem(String);

    impl raiden::IntoStringSetItem for CustomSSItem {
        fn into_ss_item(self) -> String {
            "test".to_owned()
        }
    }

    impl raiden::FromStringSetItem for CustomSSItem {
        fn from_ss_item(value: String) -> Result<Self, ConversionError> {
            Ok(CustomSSItem(value))
        }
    }

    #[derive(Raiden)]
    #[raiden(table_name = "user")]
    #[derive(Debug, Clone, PartialEq)]
    pub struct UserWithCustomStringSet {
        #[raiden(partition_key)]
        pub id: String,
        pub name: String,
        pub string_set: std::collections::HashSet<CustomSSItem>,
    }

    #[test]
    fn test_get_custom_stringset() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = UserWithCustomStringSet::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let res = client.get("user_primary_key").consistent().run().await;
            let mut set = std::collections::HashSet::new();
            set.insert(CustomSSItem("Hello".to_owned()));
            assert_eq!(
                res.unwrap(),
                get::GetOutput {
                    item: UserWithCustomStringSet {
                        id: "user_primary_key".to_owned(),
                        name: "bokuweb".to_owned(),
                        string_set: set,
                    },
                    consumed_capacity: None,
                }
            );
        }
        rt.block_on(example());
    }

    #[derive(Raiden, Debug, PartialEq)]
    #[raiden(table_name = "QueryTestData0")]
    pub struct UserWithSortKey {
        #[raiden(partition_key)]
        id: String,
        name: String,
        #[raiden(sort_key)]
        year: usize,
        num: usize,
    }

    #[test]
    fn test_user_get_item_with_sort_key() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = UserWithSortKey::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });

            let res = client.get("id1", 2003_usize).run().await;
            assert_eq!(
                res.unwrap(),
                get::GetOutput {
                    item: UserWithSortKey {
                        id: "id1".to_owned(),
                        name: "bob".to_owned(),
                        year: 2003,
                        num: 300,
                    },
                    consumed_capacity: None,
                }
            );
        }
        rt.block_on(example());
    }

    #[derive(Raiden, Debug, Clone, PartialEq)]
    pub struct EmptyStringTestData0 {
        #[raiden(partition_key)]
        #[raiden(uuid)]
        id: String,
        name: String,
    }

    #[test]
    fn test_get_empty_string() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = EmptyStringTestData0::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });

            let res = client.get("id0").run().await;
            assert_eq!(
                res.unwrap(),
                get::GetOutput {
                    item: EmptyStringTestData0 {
                        id: "id0".to_owned(),
                        name: "".to_owned(),
                    },
                    consumed_capacity: None,
                }
            );
        }
        rt.block_on(example());
    }

    #[derive(Raiden, Debug, Clone, PartialEq)]
    pub struct UseDefaultForNull {
        #[raiden(partition_key)]
        #[raiden(uuid)]
        id: String,
        #[raiden(use_default)]
        flag: bool,
        #[raiden(use_default)]
        type_param: std::collections::BTreeSet<usize>,
    }

    #[tokio::test]
    async fn test_use_default_for_null() {
        let client = UseDefaultForNull::client(Region::Custom {
            endpoint: "http://localhost:8000".into(),
            name: "ap-northeast-1".into(),
        });

        let res = client.get("id0").run().await;
        assert_eq!(
            res.unwrap(),
            get::GetOutput {
                item: UseDefaultForNull {
                    id: "id0".to_owned(),
                    flag: false,
                    type_param: Default::default(),
                },
                consumed_capacity: None,
            }
        );
    }

    use std::sync::atomic::{AtomicUsize, Ordering};

    static RETRY_COUNT: AtomicUsize = AtomicUsize::new(0);
    struct MyRetryStrategy;

    impl RetryStrategy for MyRetryStrategy {
        fn should_retry(&self, _error: &RaidenError) -> bool {
            RETRY_COUNT.fetch_add(1, Ordering::Relaxed);
            true
        }

        fn policy(&self) -> Policy {
            Policy::Limit(3)
        }
    }

    #[test]
    fn test_retry() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = User::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let _ = client
                .with_retries(Box::new(MyRetryStrategy))
                .get("anonymous")
                .run()
                .await;
        }
        rt.block_on(example());
        assert_eq!(RETRY_COUNT.load(Ordering::Relaxed), 4)
    }

    #[test]
    fn test_should_build_with_twice_retry() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = User::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            })
            .with_retries(Box::new(MyRetryStrategy));
            let _ = client.get("anonymous").run().await;
            let _ = client.get("anonymous").run().await;
        }
        rt.block_on(example());
    }

    #[derive(Raiden)]
    #[raiden(table_name = "user")]
    #[derive(Debug, Clone, PartialEq)]
    pub struct PartialUser {
        #[raiden(partition_key)]
        id: String,
        name: String,
        num_usize: usize,
    }

    #[test]
    fn test_user_get_item_for_projection_expression() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = PartialUser::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });

            let res = client.get("user_primary_key").run().await;
            assert_eq!(
                res.unwrap(),
                get::GetOutput {
                    item: PartialUser {
                        id: "user_primary_key".to_owned(),
                        name: "bokuweb".to_owned(),
                        num_usize: 42,
                    },
                    consumed_capacity: None,
                }
            );
        }
        rt.block_on(example());
    }

    #[derive(Raiden)]
    #[raiden(table_name = "ReservedTestData0")]
    #[derive(Debug, Clone, PartialEq)]
    pub struct Reserved {
        #[raiden(partition_key)]
        id: String,
        #[raiden(rename = "type")]
        r#type: String,
    }

    #[test]
    fn test_reserved_keyword() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = Reserved::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let res = client.get("id0").run().await;
            assert_eq!(
                res.unwrap(),
                get::GetOutput {
                    item: Reserved {
                        id: "id0".to_owned(),
                        r#type: "reserved".to_owned(),
                    },
                    consumed_capacity: None,
                }
            );
        }
        rt.block_on(example());
    }

    #[derive(Raiden)]
    #[raiden(table_name = "ReservedTestData0")]
    #[derive(Debug, Clone, PartialEq)]
    pub struct ReservedWithRename {
        #[raiden(partition_key)]
        id: String,
        #[raiden(rename = "type")]
        some_type: String,
    }

    #[test]
    fn test_rename_with_reserved() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = ReservedWithRename::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let res = client.get("id0").run().await;
            assert_eq!(
                res.unwrap(),
                get::GetOutput {
                    item: ReservedWithRename {
                        id: "id0".to_owned(),
                        some_type: "reserved".to_owned(),
                    },
                    consumed_capacity: None,
                }
            );
        }
        rt.block_on(example());
    }

    #[derive(Raiden)]
    #[raiden(table_name = "UseDefaultTestData0")]
    #[derive(Debug, Clone, PartialEq)]
    pub struct UseDefault {
        #[raiden(partition_key)]
        id: String,
        #[raiden(use_default)]
        is_ok: bool,
    }

    #[test]
    fn test_use_default() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = UseDefault::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let res = client.get("id0").run().await;
            assert_eq!(
                res.unwrap(),
                get::GetOutput {
                    item: UseDefault {
                        id: "id0".to_owned(),
                        is_ok: false,
                    },
                    consumed_capacity: None,
                }
            );
        }
        rt.block_on(example());
    }

    #[derive(Raiden)]
    #[raiden(table_name = "FloatTest")]
    #[derive(Debug, Clone, PartialEq)]
    pub struct FloatTest {
        #[raiden(partition_key)]
        id: String,
        float32: f32,
        float64: f64,
    }

    #[test]
    fn test_float() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = FloatTest::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let res = client.get("primary_key").run().await;
            assert_eq!(
                res.unwrap(),
                get::GetOutput {
                    item: FloatTest {
                        id: "primary_key".to_owned(),
                        float32: 1.23,
                        float64: 2.34,
                    },
                    consumed_capacity: None,
                }
            );
        }
        rt.block_on(example());
    }
}
