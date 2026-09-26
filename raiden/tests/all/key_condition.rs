#[cfg(test)]
mod tests {

    #[cfg(test)]
    use pretty_assertions::assert_eq;
    use raiden::*;

    #[allow(dead_code)]
    #[derive(Raiden)]
    #[raiden(table_name = "user")]
    #[derive(Debug, Clone)]
    pub struct User {
        #[raiden(partition_key)]
        id: String,
        name: String,
        year: usize,
        num: usize,
        #[raiden(rename = "Renamed")]
        rename: usize,
    }

    #[allow(dead_code)]
    #[derive(Raiden, Debug, Clone)]
    #[raiden(table_name = "table_key_user")]
    pub struct TableKeyUser {
        #[raiden(partition_key)]
        #[raiden(rename = "accountId")]
        account_id: String,
        #[raiden(sort_key)]
        #[raiden(rename = "createdAt")]
        created_at: String,
        name: String,
    }

    #[test]
    fn typed_partition_key_without_sort_key() {
        reset_value_id();
        let (expression, names, values) = User::partition_key_condition().eq("id1").build();
        assert_eq!(expression, "#id = :value0");
        assert_eq!(names.get("#id"), Some(&"id".to_owned()));
        assert_eq!(values.get(":value0"), Some(&"id1".into_attr()));
    }

    #[test]
    fn typed_partition_and_sort_keys_use_renamed_attributes() {
        reset_value_id();
        let condition = TableKeyUser::partition_key_condition()
            .eq("account1")
            .and(TableKeyUser::sort_key_condition().begins_with("2026-"));
        let (expression, names, values) = condition.build();
        assert_eq!(
            expression,
            "#accountId = :value0 AND (begins_with(#createdAt, :value1))"
        );
        assert_eq!(names.get("#accountId"), Some(&"accountId".to_owned()));
        assert_eq!(names.get("#createdAt"), Some(&"createdAt".to_owned()));
        assert_eq!(values.get(":value0"), Some(&"account1".into_attr()));
        assert_eq!(values.get(":value1"), Some(&"2026-".into_attr()));
    }

    #[test]
    fn test_eq_key_condition() {
        reset_value_id();
        let cond = User::key_condition(User::name()).eq("bokuweb");
        let (key_condition, attribute_names, attribute_values) = cond.build();
        let mut expected_names: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        expected_names.insert("#name".to_owned(), "name".to_owned());
        let mut expected_values: std::collections::HashMap<String, AttributeValue> =
            std::collections::HashMap::new();
        expected_values.insert(":value0".to_owned(), "bokuweb".into_attr());
        assert_eq!(key_condition, "#name = :value0".to_owned());
        assert_eq!(attribute_names, expected_names);
        assert_eq!(attribute_values, expected_values);
    }

    #[test]
    fn test_two_and_key_condition() {
        reset_value_id();

        let cond = User::key_condition(User::name()).eq("bokuweb").and(
            User::key_condition(User::year())
                .eq(1999)
                .and(User::key_condition(User::num()).eq(100)),
        );

        let (key_condition, attribute_names, attribute_values) = cond.build();
        let mut expected_names: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        expected_names.insert("#name".to_owned(), "name".to_owned());
        expected_names.insert("#year".to_owned(), "year".to_owned());
        expected_names.insert("#num".to_owned(), "num".to_owned());
        let mut expected_values: std::collections::HashMap<String, AttributeValue> =
            std::collections::HashMap::new();
        expected_values.insert(":value0".to_owned(), "bokuweb".into_attr());
        expected_values.insert(":value1".to_owned(), 1999.into_attr());
        expected_values.insert(":value2".to_owned(), 100.into_attr());

        assert_eq!(
            key_condition,
            "#name = :value0 AND (#year = :value1 AND (#num = :value2))".to_owned(),
        );
        assert_eq!(attribute_names, expected_names);
        assert_eq!(attribute_values, expected_values);
    }

    #[test]
    fn test_begins_with_key_condition() {
        reset_value_id();

        let cond = User::key_condition(User::name()).begins_with("bokuweb");
        let (key_condition, attribute_names, attribute_values) = cond.build();
        let mut expected_names: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        expected_names.insert("#name".to_owned(), "name".to_owned());
        let mut expected_values: std::collections::HashMap<String, AttributeValue> =
            std::collections::HashMap::new();
        expected_values.insert(":value0".to_owned(), "bokuweb".into_attr());
        assert_eq!(key_condition, "begins_with(#name, :value0)".to_owned());
        assert_eq!(attribute_names, expected_names);
        assert_eq!(attribute_values, expected_values);
    }

    #[test]
    fn test_begins_with_id_and_key_condition() {
        reset_value_id();

        let cond = User::key_condition(User::id())
            .eq("id3")
            .and(User::key_condition(User::year()).begins_with("20"));
        let (key_condition, _attribute_names, _attribute_values) = cond.build();
        assert_eq!(
            key_condition,
            "#id = :value0 AND (begins_with(#year, :value1))".to_owned(),
        );
    }
}
