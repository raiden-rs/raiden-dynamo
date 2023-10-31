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
    fn test_update() {
        async fn example() {
            let client = crate::all::create_client_from_struct!(User);
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

        tokio::runtime::Runtime::new().unwrap().block_on(example());
    }

    #[test]
    fn test_update_with_invalid_key_with_condition() {
        async fn example() {
            let client = crate::all::create_client_from_struct!(User);
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

        tokio::runtime::Runtime::new().unwrap().block_on(example());
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
        async fn example() {
            let client = crate::all::create_client_from_struct!(UserWithUnStored);
            let set_expression = UserWithUnStored::update_expression()
                .set(UserWithUnStored::name())
                .value("updated!!");
            let res = client
                .update("id0")
                .set(set_expression)
                .run()
                .await
                .unwrap();

            assert_eq!(res.item, None);
        }

        tokio::runtime::Runtime::new().unwrap().block_on(example());
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
        async fn example() {
            let client = crate::all::create_client_from_struct!(UpdateTestData1);
            let set_expression = UpdateTestData1::update_expression()
                .set(UpdateTestData1::name())
                .value("bob");
            let res = client
                .update("id0", 36_usize)
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

        tokio::runtime::Runtime::new().unwrap().block_on(example());
    }

    #[derive(Raiden, Debug, Clone, PartialEq)]
    pub struct EmptySetTestData0 {
        #[raiden(partition_key)]
        id: String,
        sset: std::collections::HashSet<String>,
    }

    #[test]
    fn test_update_with_only_attr() {
        async fn example() {
            let client = crate::all::create_client_from_struct!(EmptySetTestData0);
            let set_expression = EmptySetTestData0::update_expression()
                .set(EmptySetTestData0::sset())
                .attr(EmptySetTestData0::sset());
            let res = client
                .update("id1")
                .set(set_expression)
                .return_all_new()
                .run()
                .await;

            assert_eq!(res.is_ok(), true);
        }

        tokio::runtime::Runtime::new().unwrap().block_on(example());
    }

    #[test]
    fn test_update_empty_set_sort_key() {
        async fn example() {
            let client = crate::all::create_client_from_struct!(EmptySetTestData0);
            let sset: std::collections::HashSet<String> = std::collections::HashSet::new();
            let expected_sset: std::collections::HashSet<String> = std::collections::HashSet::new();
            let set_expression = EmptySetTestData0::update_expression()
                .set(EmptySetTestData0::sset())
                .value(sset);
            let res = client
                .update("id0")
                .set(set_expression)
                .return_all_new()
                .run()
                .await;

            assert!(res.is_ok());
            assert_eq!(res.unwrap().item.unwrap().sset, expected_sset);
        }

        tokio::runtime::Runtime::new().unwrap().block_on(example());
    }

    #[test]
    fn test_add_with_empty_hash_set() {
        async fn example() {
            let client = crate::all::create_client_from_struct!(EmptySetTestData0);
            let sset: std::collections::HashSet<String> = std::collections::HashSet::new();
            let mut expected_sset: std::collections::HashSet<String> =
                std::collections::HashSet::new();
            expected_sset.insert("Hello".to_owned());
            let expression = EmptySetTestData0::update_expression()
                .add(EmptySetTestData0::sset())
                .value(sset);
            let res = client
                .update("id0")
                .add(expression)
                .return_all_new()
                .run()
                .await;

            assert_eq!(res.is_ok(), true);
            assert_eq!(res.unwrap().item.unwrap().sset, expected_sset);
        }

        tokio::runtime::Runtime::new().unwrap().block_on(example());
    }

    #[derive(Raiden, Debug, Clone, PartialEq)]
    pub struct UpdateDeleteTestData0 {
        #[raiden(partition_key)]
        id: String,
        sset: std::collections::HashSet<String>,
    }

    #[test]
    fn test_update_delete_sset() {
        async fn example() {
            let client = crate::all::create_client_from_struct!(UpdateDeleteTestData0);
            let mut delete_sset: std::collections::HashSet<String> =
                std::collections::HashSet::new();
            delete_sset.insert("foo".to_owned());
            let delete_expression = UpdateDeleteTestData0::update_expression()
                .delete(UpdateDeleteTestData0::sset())
                .value(delete_sset);
            let mut expected_sset: std::collections::HashSet<String> =
                std::collections::HashSet::new();
            expected_sset.insert("bar".to_owned());

            let res = client
                .update("id0")
                .delete(delete_expression)
                .return_all_new()
                .run()
                .await;

            assert_eq!(res.unwrap().item.unwrap().sset, expected_sset);
        }

        tokio::runtime::Runtime::new().unwrap().block_on(example());
    }

    #[derive(Raiden, Debug, Clone, PartialEq)]
    pub struct UpdateAddTestData0 {
        #[raiden(partition_key)]
        id: String,
        sset: std::collections::HashSet<String>,
    }

    #[test]
    fn test_update_add_sset() {
        async fn example() {
            let client = crate::all::create_client_from_struct!(UpdateAddTestData0);
            let mut add_sset: std::collections::HashSet<String> = std::collections::HashSet::new();
            add_sset.insert("added".to_owned());
            let add_expression = UpdateAddTestData0::update_expression()
                .add(UpdateAddTestData0::sset())
                .value(add_sset);
            let mut expected_sset: std::collections::HashSet<String> =
                std::collections::HashSet::new();
            expected_sset.insert("foo".to_owned());
            expected_sset.insert("bar".to_owned());
            expected_sset.insert("added".to_owned());

            let res = client
                .update("id0")
                .add(add_expression)
                .return_all_new()
                .run()
                .await;

            assert_eq!(res.unwrap().item.unwrap().sset, expected_sset);
        }

        tokio::runtime::Runtime::new().unwrap().block_on(example());
    }

    #[test]
    fn test_update_add_sset_to_empty() {
        async fn example() {
            let client = crate::all::create_client_from_struct!(UpdateAddTestData0);
            let mut add_sset: std::collections::HashSet<String> = std::collections::HashSet::new();
            add_sset.insert("added".to_owned());
            let add_expression = UpdateAddTestData0::update_expression()
                .add(UpdateAddTestData0::sset())
                .value(add_sset);
            let mut expected_sset: std::collections::HashSet<String> =
                std::collections::HashSet::new();
            expected_sset.insert("added".to_owned());

            let res = client
                .update("id2")
                .add(add_expression)
                .return_all_new()
                .run()
                .await;

            assert_eq!(res.unwrap().item.unwrap().sset, expected_sset);
        }

        tokio::runtime::Runtime::new().unwrap().block_on(example());
    }

    #[test]
    fn test_update_to_empty_string() {
        async fn example() {
            let client = crate::all::create_client_from_struct!(User);
            let set_name_expression = User::update_expression().set(User::name()).value("");
            let res = client
                .update("id0")
                .set(set_name_expression)
                .return_all_new()
                .run()
                .await
                .unwrap();

            assert_eq!(res.item.unwrap().name, "".to_owned());
        }

        tokio::runtime::Runtime::new().unwrap().block_on(example());
    }

    #[derive(Raiden, Debug, Clone, PartialEq)]
    pub struct UpdateRemoveTestData0 {
        #[raiden(partition_key)]
        id: String,
        name: Option<String>,
    }

    #[test]
    fn test_update_remove_sset() {
        async fn example() {
            let client = crate::all::create_client_from_struct!(UpdateRemoveTestData0);
            let res = client
                .update("id1")
                .remove(UpdateRemoveTestData0::name())
                .return_all_new()
                .run()
                .await;

            assert_eq!(
                res.unwrap().item.unwrap(),
                UpdateRemoveTestData0 {
                    id: "id1".to_owned(),
                    name: None
                }
            );

            let res = client
                .update("id2")
                .remove(UpdateRemoveTestData0::name())
                .return_all_new()
                .run()
                .await;

            assert_eq!(
                res.unwrap().item.unwrap(),
                UpdateRemoveTestData0 {
                    id: "id2".to_owned(),
                    name: None
                }
            );
        }

        tokio::runtime::Runtime::new().unwrap().block_on(example());
    }

    #[derive(Raiden, Debug, Clone, PartialEq)]
    pub struct UpdateWithContainsInSetCondition {
        #[raiden(partition_key)]
        id: String,
        sset: std::collections::HashSet<String>,
        name: String,
    }

    #[test]
    fn should_update_with_contains_condition_in_sset() {
        async fn example() {
            let client = crate::all::create_client_from_struct!(UpdateWithContainsInSetCondition);
            let set_expression = UpdateWithContainsInSetCondition::update_expression()
                .set(UpdateWithContainsInSetCondition::name())
                .value("Changed");
            let cond = UpdateWithContainsInSetCondition::condition().contains(
                UpdateWithContainsInSetCondition::sset(),
                "Hello".to_string(),
            );

            let res = client
                .update("id0")
                .set(set_expression)
                .condition(cond)
                .return_all_new()
                .run()
                .await;

            assert_eq!(res.is_ok(), true);
        }

        tokio::runtime::Runtime::new().unwrap().block_on(example());
    }

    #[test]
    fn should_not_update_with_contains_condition_in_sset() {
        async fn example() {
            let client = crate::all::create_client_from_struct!(UpdateWithContainsInSetCondition);
            let set_expression = UpdateWithContainsInSetCondition::update_expression()
                .set(UpdateWithContainsInSetCondition::name())
                .value("Changed");
            let cond = UpdateWithContainsInSetCondition::condition().contains(
                UpdateWithContainsInSetCondition::sset(),
                "World".to_string(),
            );

            let res = client
                .update("id0")
                .set(set_expression)
                .condition(cond)
                .return_all_new()
                .run()
                .await;

            assert_eq!(res.is_err(), true);
        }

        tokio::runtime::Runtime::new().unwrap().block_on(example());
    }

    #[test]
    fn should_not_update_with_or_condition_failed() {
        async fn example() {
            let client = crate::all::create_client_from_struct!(UpdateWithContainsInSetCondition);
            let set_expression = UpdateWithContainsInSetCondition::update_expression()
                .set(UpdateWithContainsInSetCondition::name())
                .value("Changed");
            let cond = UpdateWithContainsInSetCondition::condition()
                .contains(
                    UpdateWithContainsInSetCondition::sset(),
                    "Merhaba".to_string(),
                )
                .or(UpdateWithContainsInSetCondition::condition().contains(
                    UpdateWithContainsInSetCondition::sset(),
                    "Bonjour".to_string(),
                ));

            let res = client
                .update("id0")
                .set(set_expression)
                .condition(cond)
                .return_all_new()
                .run()
                .await;

            assert_eq!(res.is_err(), true);
        }

        tokio::runtime::Runtime::new().unwrap().block_on(example());
    }

    #[test]
    fn should_update_with_or_condition() {
        async fn example() {
            let client = crate::all::create_client_from_struct!(UpdateWithContainsInSetCondition);
            let set_expression = UpdateWithContainsInSetCondition::update_expression()
                .set(UpdateWithContainsInSetCondition::name())
                .value("Changed");
            let cond = UpdateWithContainsInSetCondition::condition()
                .contains(
                    UpdateWithContainsInSetCondition::sset(),
                    "Hello".to_string(),
                )
                .or(UpdateWithContainsInSetCondition::condition().contains(
                    UpdateWithContainsInSetCondition::sset(),
                    "Bonjour".to_string(),
                ));

            let res = client
                .update("id0")
                .set(set_expression)
                .condition(cond)
                .return_all_new()
                .run()
                .await;

            assert_eq!(res.is_ok(), true);
        }

        tokio::runtime::Runtime::new().unwrap().block_on(example());
    }

    #[test]
    fn should_set_if_attr_not_exists() {
        async fn example() {
            let client = crate::all::create_client_from_struct!(User);
            let set_name_expression = User::update_expression()
                .set(User::name())
                .value("updated")
                .if_not_exists();
            let set_age_expression = User::update_expression().set(User::age()).value(12);

            let res = client
                .update("if_not_exists#0")
                .set(set_name_expression)
                .set(set_age_expression)
                .return_all_new()
                .run()
                .await
                .unwrap();

            assert_eq!(
                res.item,
                Some(User {
                    id: "if_not_exists#0".into(),
                    name: "updated".into(),
                    age: 12
                })
            );
        }

        tokio::runtime::Runtime::new().unwrap().block_on(example());
    }

    #[test]
    fn should_not_set_if_attr_exists() {
        async fn example() {
            let client = crate::all::create_client_from_struct!(User);

            let set_name_expression = User::update_expression().set(User::name()).value("created");
            let set_age_expression = User::update_expression().set(User::age()).value(12);

            client
                .update("if_not_exists#1")
                .set(set_name_expression)
                .set(set_age_expression)
                .run()
                .await
                .unwrap();

            let set_name_expression = User::update_expression()
                .set(User::name())
                .value("updated")
                .if_not_exists(); // update only if attribute not exists
            let res = client
                .update("if_not_exists#1")
                .set(set_name_expression)
                .return_all_new()
                .run()
                .await
                .unwrap();
            assert_eq!(res.item.map(|u| u.name), Some("created".into()));
        }

        tokio::runtime::Runtime::new().unwrap().block_on(example());
    }
}
