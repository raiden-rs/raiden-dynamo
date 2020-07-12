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
        age: usize,
    }

    #[test]
    fn test_minimum_set_update_expression() {
        let client = User::client(Region::Custom {
            endpoint: "http://localhost:8000".into(),
            name: "ap-northeast-1".into(),
        });
        reset_value_id();
        let (expression, _, _) = client
            .update("id0")
            .set(User::update_expression().set(User::name()).value("updated"))
            .build_expression();

        assert_eq!(expression, "SET #name = :value0".to_owned());
    }

    #[test]
    fn test_set_and_add_update_expression() {
        let client = User::client(Region::Custom {
            endpoint: "http://localhost:8000".into(),
            name: "ap-northeast-1".into(),
        });
        reset_value_id();
        let (expression, _, _) = client
            .update("id0")
            .set(User::update_expression().set(User::name()).value("updated"))
            .add(User::update_expression().add(User::age()).value(1))
            .build_expression();

        assert_eq!(
            expression,
            "ADD #age :value1 SET #name = :value0".to_owned()
        );
    }

    #[test]
    fn test_update() {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = User::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let set_name_expression = User::update_expression()
                .set(User::name())
                .value("updated!!");
            let set_age_expression = User::update_expression().set(User::age()).value(12);
            let res = client
                .update("id0")
                .set(set_name_expression)
                .set(set_age_expression)
                .return_all_new()
                .run()
                .await
                .unwrap();
            assert_eq!(
                res.item,
                Some(User {
                    id: "id0".to_owned(),
                    name: "updated!!".to_owned(),
                    age: 12,
                })
            );
        }
        rt.block_on(example());
    }

    #[test]
    fn test_update_with_invalid_key_with_condition() {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = User::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let cond = User::condition().attr_exists(User::id());
            let set_expression = User::update_expression()
                .set(User::name())
                .value("updated!!");
            let res = client
                .update("invalid_key!!!!!!")
                .return_all_new()
                .condition(cond)
                .set(set_expression)
                .run()
                .await;
            assert_eq!(res.is_err(), true);
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
    fn test_update_with_unstored() {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = UserWithUnStored::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let set_expression = UserWithUnStored::update_expression()
                .set(UserWithUnStored::name())
                .value("updated!!");
            let res = client
                .update("id0")
                .set(set_expression)
                .run()
                .await
                .unwrap();
            assert_eq!(res.item, None,);
        }
        rt.block_on(example());
    }

    #[derive(Raiden, Debug, Clone, PartialEq)]
    pub struct UpdateTestData1 {
        #[raiden(partition_key)]
        id: String,
        name: String,
        #[raiden(sort_key)]
        age: usize,
    }

    #[test]
    fn test_update_with_sort_key() {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = UpdateTestData1::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let set_expression = UpdateTestData1::update_expression()
                .set(UpdateTestData1::name())
                .value("bob");
            let res = client
                .update("id0", 36)
                .set(set_expression)
                .return_all_new()
                .run()
                .await
                .unwrap();
            assert_eq!(
                res.item,
                Some(UpdateTestData1 {
                    id: "id0".to_owned(),
                    name: "bob".to_owned(),
                    age: 36
                })
            );
        }
        rt.block_on(example());
    }
}
