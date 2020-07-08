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
    fn test_update() {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = User::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let set_expression = User::update_expression()
                .set(User::name())
                .value("updated!!");
            let res = client
                .update("id0")
                .set(set_expression)
                .return_all_new()
                .run()
                .await
                .unwrap();
            assert_eq!(
                res.item,
                Some(User {
                    id: "id0".to_owned(),
                    name: "updated!!".to_owned()
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
            assert_eq!(
                res.item,
                None,
            );
        }
        rt.block_on(example());
    }
}
