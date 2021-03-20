#[cfg(test)]
mod tests {

    #[cfg(test)]
    use pretty_assertions::assert_eq;
    use raiden::*;

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
        assert_eq!(key_condition, "#name = :value0".to_owned(),);
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
        assert_eq!(key_condition, "begins_with(#name, :value0)".to_owned(),);
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
