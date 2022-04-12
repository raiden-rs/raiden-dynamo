#[cfg(test)]
mod tests {

    #[cfg(test)]
    use pretty_assertions::assert_eq;
    use raiden::*;

    #[derive(Raiden)]
    #[raiden(table_name = "user")]
    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    pub struct User {
        #[raiden(partition_key)]
        id: String,
        name: String,
    }

    #[derive(Raiden)]
    #[raiden(table_name = "user")]
    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    pub struct UserWithUuid {
        #[raiden(partition_key)]
        #[raiden(uuid)]
        id: String,
        name: String,
    }

    #[test]
    fn test_put_user_with_attribute_not_exists_condition_input() {
        let client = User::client(Region::Custom {
            endpoint: "http://localhost:8000".into(),
            name: "ap-northeast-1".into(),
        });
        let user = UserPutItemInput {
            id: "mock_id".to_owned(),
            name: "bokuweb".to_owned(),
        };
        let cond = User::condition().attr_not_exists(User::name());
        let input = client.put(user).condition(cond).input;
        let mut attribute_names: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        let mut item: std::collections::HashMap<String, raiden::AttributeValue> =
            std::collections::HashMap::new();

        item.insert(
            "name".to_owned(),
            raiden::AttributeValue {
                s: Some("bokuweb".to_owned()),
                ..raiden::AttributeValue::default()
            },
        );
        item.insert(
            "id".to_owned(),
            raiden::AttributeValue {
                s: Some("mock_id".to_owned()),
                ..raiden::AttributeValue::default()
            },
        );
        attribute_names.insert("#name".to_owned(), "name".to_owned());
        assert_eq!(
            input,
            ::raiden::PutItemInput {
                condition_expression: Some("attribute_not_exists(#name)".to_owned()),
                expression_attribute_names: Some(attribute_names),
                expression_attribute_values: None,
                item,
                table_name: "user".to_owned(),
                ..::raiden::PutItemInput::default()
            },
        );
    }

    #[test]
    fn test_put_user() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = User::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let user = UserPutItemInput {
                id: "mock_id".to_owned(),
                name: "bokuweb".to_owned(),
            };
            let _res = client.put(user).run().await;
        }
        rt.block_on(example());
    }

    #[test]
    fn test_put_user_with_builder() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = User::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let user = User::put_item_builder()
                .id("mock_id".to_owned())
                .name("bokuweb".to_owned())
                .build();
            let _res = client.put(user).run().await;
        }
        rt.block_on(example());
    }

    #[derive(Raiden, Debug, Clone)]
    #[allow(dead_code)]
    pub struct PutItemConditionData0 {
        #[raiden(partition_key)]
        id: String,
        name: String,
    }

    #[test]
    fn test_put_user_eq_op_condition_expression() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = User::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let user = UserPutItemInput {
                id: "id0".to_owned(),
                name: "bokuweb".to_owned(),
            };
            let cond = User::condition().value("bokuweb").eq_attr(User::name());
            let res = client.put(user).condition(cond).run().await;
            assert_eq!(res.is_ok(), true);
        }
        rt.block_on(example());
    }

    #[test]
    #[allow(dead_code)]
    fn test_put_user_eq_op_condition_expression_with_not_exist_name() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = User::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let user = UserPutItemInput {
                id: "id0".to_owned(),
                name: "bokuweb".to_owned(),
            };
            let cond = User::condition().value("bokuweb_").eq_attr(User::name());
            let res = client.put(user).condition(cond).run().await;
            assert_eq!(
                Err(::raiden::RaidenError::ConditionalCheckFailed(
                    "The conditional request failed".to_owned()
                )),
                res
            );
        }
        rt.block_on(example());
    }

    #[test]
    #[allow(dead_code)]
    fn test_put_user_id_not_exists_expression() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = User::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let user = UserPutItemInput {
                id: "id0".to_owned(),
                name: "bokuweb".to_owned(),
            };
            let cond = User::condition().attr_not_exists(User::id());
            let res = client.put(user).condition(cond).run().await;
            assert_eq!(
                Err(::raiden::RaidenError::ConditionalCheckFailed(
                    "The conditional request failed".to_owned()
                )),
                res
            );
        }
        rt.block_on(example());
    }

    #[test]
    #[allow(dead_code)]
    fn test_put_user_id_exists_expression() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = User::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let user = UserPutItemInput {
                id: "id0".to_owned(),
                name: "bokuweb".to_owned(),
            };
            let cond = User::condition().attr_exists(User::id());
            let res = client.put(user).condition(cond).run().await;
            assert_eq!(res.is_ok(), true);
        }
        rt.block_on(example());
    }

    #[test]
    #[allow(dead_code)]
    fn test_put_user_with_uuid() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = UserWithUuid::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let item = UserWithUuid::put_item_builder()
                .name("bokuweb".to_owned())
                .build();
            let res = client.put(item).run().await;
            assert_eq!(res.is_ok(), true);
        }
        rt.block_on(example());
    }

    #[derive(Raiden)]
    #[raiden(table_name = "user")]
    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    pub struct UserVecTest {
        #[raiden(partition_key)]
        #[raiden(uuid)]
        id: String,
        name: String,
        nums: Vec<usize>,
    }

    #[test]
    fn test_put_user_with_number_vec() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = UserVecTest::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let item = UserVecTest::put_item_builder()
                .name("bokuweb".to_owned())
                .nums(vec![0, 1, 2])
                .build();
            let res = client.put(item).run().await;
            assert_eq!(res.is_ok(), true);
        }
        rt.block_on(example());
    }

    #[derive(Raiden)]
    #[raiden(table_name = "user")]
    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    pub struct UserSetTest {
        #[raiden(partition_key)]
        #[raiden(uuid)]
        id: String,
        name: String,
        nums: std::collections::HashSet<usize>,
    }

    #[test]
    fn test_put_user_with_number_set() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = UserSetTest::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let mut nums: std::collections::HashSet<usize> = std::collections::HashSet::new();
            nums.insert(1);

            let item = UserSetTest::put_item_builder()
                .name("bokuweb".to_owned())
                .nums(nums)
                .build();
            let res = client.put(item).run().await;
            assert_eq!(res.is_ok(), true);
        }
        rt.block_on(example());
    }

    #[derive(Debug, Clone, Hash, PartialEq, Eq)]
    pub struct Custom {}

    impl raiden::IntoStringSetItem for Custom {
        fn into_ss_item(self) -> String {
            "test".to_owned()
        }
    }

    impl raiden::FromStringSetItem for Custom {
        fn from_ss_item(_value: String) -> Result<Self, ConversionError> {
            Ok(Custom {})
        }
    }

    #[derive(Raiden)]
    #[raiden(table_name = "user")]
    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    pub struct UserDefinedSetTest {
        #[raiden(partition_key)]
        #[raiden(uuid)]
        id: String,
        name: String,
        nums: std::collections::HashSet<Custom>,
    }

    #[test]
    fn test_put_user_with_user_defined_set() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = UserSetTest::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let mut nums: std::collections::HashSet<usize> = std::collections::HashSet::new();
            nums.insert(1);

            let item = UserSetTest::put_item_builder()
                .name("bokuweb".to_owned())
                .nums(nums)
                .build();
            let res = client.put(item).run().await;
            assert_eq!(res.is_ok(), true);
        }
        rt.block_on(example());
    }

    #[derive(Raiden)]
    #[raiden(table_name = "user")]
    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    pub struct UserEmptySetTest {
        #[raiden(partition_key)]
        #[raiden(uuid)]
        id: String,
        set: std::collections::HashSet<String>,
    }

    #[test]
    fn test_put_user_with_empty_set() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = UserEmptySetTest::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let set: std::collections::HashSet<String> = std::collections::HashSet::new();

            let item = UserEmptySetTest::put_item_builder().set(set).build();
            let res = client.put(item).run().await;
            assert_eq!(res.is_ok(), true);
        }
        rt.block_on(example());
    }

    #[derive(Raiden, Debug, Clone)]
    #[allow(dead_code)]
    pub struct EmptyStringTestData0 {
        #[raiden(partition_key)]
        #[raiden(uuid)]
        id: String,
        name: String,
    }

    #[test]
    fn test_put_with_empty_string() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = EmptyStringTestData0::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let item = EmptyStringTestData0::put_item_builder()
                .name("".to_owned())
                .build();
            let res = client.put(item).run().await;
            assert_eq!(res.is_ok(), true);
        }
        rt.block_on(example());
    }

    #[derive(Raiden, Debug, Clone, PartialEq)]
    #[allow(dead_code)]
    pub struct EmptyPutTestData0 {
        #[raiden(partition_key)]
        id: String,
        sset: std::collections::HashSet<String>,
    }

    #[test]
    fn test_put_with_empty_sset() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = EmptyPutTestData0::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let set: std::collections::HashSet<String> = std::collections::HashSet::new();
            let expected_set: std::collections::HashSet<String> = std::collections::HashSet::new();
            let item = EmptyPutTestData0::put_item_builder()
                .id("testid".to_owned())
                .sset(set)
                .build();
            let res = client.put(item).run().await;
            assert_eq!(res.is_ok(), true);
            let res = client.get(res.unwrap().item.id).run().await;
            assert_eq!(
                res.unwrap().item,
                EmptyPutTestData0 {
                    id: "testid".to_owned(),
                    sset: expected_set
                }
            );
        }
        rt.block_on(example());
    }
}
