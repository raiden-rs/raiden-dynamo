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

    #[tokio::test]
    async fn test_user_get_item() {
        let client = crate::all::create_client_from_struct!(User);
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

    #[tokio::test]
    async fn test_user_get_item_with_consistent_read() {
        let client = crate::all::create_client!(UserClient);
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

    #[tokio::test]
    async fn test_user_get_item_with_not_found_error() {
        let client = crate::all::create_client!(UserClient);
        let res = client.get("not_exist_key").consistent().run().await;

        assert!(res.is_err());

        if let RaidenError::ResourceNotFound(msg) = res.unwrap_err() {
            assert_eq!("resource not found", msg);
        } else {
            panic!("err should be RaidenError::ResourceNotFound");
        }
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

    #[tokio::test]
    async fn test_get_unstored_value() {
        let client = crate::all::create_client_from_struct!(UserWithUnStored);
        let res = client.get("user_primary_key").consistent().run().await;

        assert!(res.is_err());

        if let RaidenError::AttributeConvertError { attr_name } = res.unwrap_err() {
            assert_eq!("unstored", attr_name);
        } else {
            panic!("err should be RaidenError::AttributeConvertError");
        }
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

    #[tokio::test]
    async fn test_get_empty_hashset() {
        let client = crate::all::create_client_from_struct!(UserWithEmptyHashSet);
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

    #[derive(Raiden)]
    #[raiden(table_name = "user")]
    #[derive(Debug, Clone, PartialEq)]
    pub struct UserWithEmptyVec {
        #[raiden(partition_key)]
        id: String,
        name: String,
        empty_vec: Vec<usize>,
    }

    #[tokio::test]
    async fn test_get_empty_vec() {
        let client = crate::all::create_client_from_struct!(UserWithEmptyVec);
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

    #[derive(Raiden)]
    #[raiden(table_name = "user")]
    #[derive(Debug, Clone, PartialEq)]
    pub struct UserWithStringSet {
        #[raiden(partition_key)]
        id: String,
        name: String,
        string_set: std::collections::HashSet<String>,
    }

    #[tokio::test]
    async fn test_get_stringset() {
        let client = crate::all::create_client_from_struct!(UserWithStringSet);
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

    #[derive(Raiden)]
    #[raiden(table_name = "user")]
    #[derive(Debug, Clone, PartialEq)]
    pub struct UserWithStringBTreeSet {
        #[raiden(partition_key)]
        id: String,
        name: String,
        string_set: std::collections::BTreeSet<String>,
    }

    #[tokio::test]
    async fn test_get_btree_stringset() {
        let client = crate::all::create_client_from_struct!(UserWithStringBTreeSet);
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

    #[tokio::test]
    async fn test_get_custom_stringset() {
        let client = crate::all::create_client_from_struct!(UserWithCustomStringSet);
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

    #[tokio::test]
    async fn test_user_get_item_with_sort_key() {
        let client = crate::all::create_client_from_struct!(UserWithSortKey);
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

    #[derive(Raiden, Debug, Clone, PartialEq)]
    pub struct EmptyStringTestData0 {
        #[raiden(partition_key)]
        #[raiden(uuid)]
        id: String,
        name: String,
    }

    #[tokio::test]
    async fn test_get_empty_string() {
        let client = crate::all::create_client_from_struct!(EmptyStringTestData0);
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
        let client = crate::all::create_client_from_struct!(UseDefaultForNull);
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

    #[tokio::test]
    async fn test_retry() {
        let client = crate::all::create_client_from_struct!(User);
        let _ = client
            .with_retries(Box::new(MyRetryStrategy))
            .get("anonymous")
            .run()
            .await;

        assert_eq!(RETRY_COUNT.load(Ordering::Relaxed), 4)
    }

    #[tokio::test]
    async fn test_should_build_with_twice_retry() {
        let client =
            crate::all::create_client_from_struct!(User).with_retries(Box::new(MyRetryStrategy));
        let _ = client.get("anonymous").run().await;
        let _ = client.get("anonymous").run().await;
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

    #[tokio::test]
    async fn test_user_get_item_for_projection_expression() {
        let client = crate::all::create_client_from_struct!(PartialUser);
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

    #[derive(Raiden)]
    #[raiden(table_name = "ReservedTestData0")]
    #[derive(Debug, Clone, PartialEq)]
    pub struct Reserved {
        #[raiden(partition_key)]
        id: String,
        #[raiden(rename = "type")]
        r#type: String,
    }

    #[tokio::test]
    async fn test_reserved_keyword() {
        let client = crate::all::create_client_from_struct!(Reserved);
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

    #[derive(Raiden)]
    #[raiden(table_name = "ReservedTestData0")]
    #[derive(Debug, Clone, PartialEq)]
    pub struct ReservedWithRename {
        #[raiden(partition_key)]
        id: String,
        #[raiden(rename = "type")]
        some_type: String,
    }

    #[tokio::test]
    async fn test_rename_with_reserved() {
        let client = crate::all::create_client_from_struct!(ReservedWithRename);
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

    #[derive(Raiden)]
    #[raiden(table_name = "UseDefaultTestData0")]
    #[derive(Debug, Clone, PartialEq)]
    pub struct UseDefault {
        #[raiden(partition_key)]
        id: String,
        #[raiden(use_default)]
        is_ok: bool,
    }

    #[tokio::test]
    async fn test_use_default() {
        let client = crate::all::create_client_from_struct!(UseDefault);
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

    #[derive(Raiden)]
    #[raiden(table_name = "FloatTest")]
    #[derive(Debug, Clone, PartialEq)]
    pub struct FloatTest {
        #[raiden(partition_key)]
        id: String,
        float32: f32,
        float64: f64,
    }

    #[tokio::test]
    async fn test_float() {
        let client = crate::all::create_client_from_struct!(FloatTest);
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
}
