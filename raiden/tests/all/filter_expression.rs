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
    fn test_eq_filter_expression() {
        reset_value_id();
        let cond = User::filter_expression(User::name()).eq("bokuweb");
        let (filter_expression, attribute_names, attribute_values) = cond.build();
        let mut expected_names: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        expected_names.insert("#name".to_owned(), "name".to_owned());
        let mut expected_values: std::collections::HashMap<String, AttributeValue> =
            std::collections::HashMap::new();
        expected_values.insert(":value0".to_owned(), "bokuweb".into_attr());
        assert_eq!(filter_expression, "#name = :value0".to_owned(),);
        assert_eq!(attribute_names, expected_names);
        assert_eq!(attribute_values, expected_values);
    }

    #[test]
    fn test_size_filter_expression() {
        reset_value_id();
        let cond = User::filter_expression(User::name()).size().eq(7);
        let (filter_expression, attribute_names, attribute_values) = cond.build();
        let mut expected_names: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        expected_names.insert("#name".to_owned(), "name".to_owned());
        let mut expected_values: std::collections::HashMap<String, AttributeValue> =
            std::collections::HashMap::new();
        expected_values.insert(":value0".to_owned(), 7.into_attr());
        assert_eq!(filter_expression, "size(#name) = :value0".to_owned(),);
        assert_eq!(attribute_names, expected_names);
        assert_eq!(attribute_values, expected_values);
    }

    #[test]
    fn test_not_filter_expression() {
        reset_value_id();
        let cond = User::filter_expression(User::name()).not("raiden");
        let (filter_expression, attribute_names, attribute_values) = cond.build();
        let mut expected_names: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        expected_names.insert("#name".to_owned(), "name".to_owned());
        let mut expected_values: std::collections::HashMap<String, AttributeValue> =
            std::collections::HashMap::new();
        expected_values.insert(":value0".to_owned(), "raiden".into_attr());

        assert_eq!(filter_expression, "#name <> :value0".to_owned(),);
        assert_eq!(attribute_names, expected_names);
        assert_eq!(attribute_values, expected_values);
    }

    #[test]
    fn test_two_and_filter_expression() {
        reset_value_id();

        let cond = User::filter_expression(User::name()).eq("bokuweb").and(
            User::filter_expression(User::year())
                .eq(1999)
                .and(User::filter_expression(User::num()).eq(100)),
        );

        let (filter_expression, attribute_names, attribute_values) = cond.build();
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
            filter_expression,
            "#name = :value0 AND (#year = :value1 AND (#num = :value2))".to_owned(),
        );
        assert_eq!(attribute_names, expected_names);
        assert_eq!(attribute_values, expected_values);
    }

    #[test]
    fn test_two_or_filter_expression() {
        reset_value_id();

        let cond = User::filter_expression(User::name())
            .eq("bokuweb")
            .or(User::filter_expression(User::year())
                .eq(1999)
                .or(User::filter_expression(User::num()).eq(100)));

        let (filter_expression, attribute_names, attribute_values) = cond.build();
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
            filter_expression,
            "#name = :value0 OR (#year = :value1 OR (#num = :value2))".to_owned(),
        );
        assert_eq!(attribute_names, expected_names);
        assert_eq!(attribute_values, expected_values);
    }

    #[test]
    fn test_begins_with_filter_expression() {
        reset_value_id();

        let cond = User::filter_expression(User::name()).begins_with("bokuweb");
        let (filter_expression, attribute_names, attribute_values) = cond.build();
        let mut expected_names: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        expected_names.insert("#name".to_owned(), "name".to_owned());
        let mut expected_values: std::collections::HashMap<String, AttributeValue> =
            std::collections::HashMap::new();
        expected_values.insert(":value0".to_owned(), "bokuweb".into_attr());
        assert_eq!(filter_expression, "begins_with(#name, :value0)".to_owned(),);
        assert_eq!(attribute_names, expected_names);
        assert_eq!(attribute_values, expected_values);
    }

    #[test]
    fn test_begins_with_id_and_filter_expression() {
        reset_value_id();

        let cond = User::filter_expression(User::id())
            .not("id3")
            .and(User::filter_expression(User::year()).begins_with("20"));
        let (filter_expression, _attribute_names, _attribute_values) = cond.build();
        assert_eq!(
            filter_expression,
            "#id <> :value0 AND (begins_with(#year, :value1))".to_owned(),
        );
    }

    #[test]
    fn test_attribute_exists_filter_expression() {
        reset_value_id();

        let cond = User::filter_expression(User::name()).attribute_exists();
        let (filter_expression, attribute_names, attribute_values) = cond.build();
        let mut expected_names: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        expected_names.insert("#name".to_owned(), "name".to_owned());
        let expected_values: std::collections::HashMap<String, AttributeValue> =
            std::collections::HashMap::new();
        assert_eq!(filter_expression, "attribute_exists(#name)".to_owned(),);
        assert_eq!(attribute_names, expected_names);
        assert_eq!(attribute_values, expected_values);
    }

    #[test]
    fn test_attribute_not_exists_filter_expression() {
        reset_value_id();

        let cond = User::filter_expression(User::name()).attribute_not_exists();
        let (filter_expression, attribute_names, attribute_values) = cond.build();
        let mut expected_names: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        expected_names.insert("#name".to_owned(), "name".to_owned());
        let expected_values: std::collections::HashMap<String, AttributeValue> =
            std::collections::HashMap::new();
        assert_eq!(filter_expression, "attribute_not_exists(#name)".to_owned(),);
        assert_eq!(attribute_names, expected_names);
        assert_eq!(attribute_values, expected_values);
    }

    #[test]
    fn test_attribute_type_filter_expression() {
        reset_value_id();

        let cond = User::filter_expression(User::name()).attribute_type(raiden::AttributeType::S);
        let (filter_expression, attribute_names, attribute_values) = cond.build();
        let mut expected_names: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        expected_names.insert("#name".to_owned(), "name".to_owned());
        let mut expected_values: std::collections::HashMap<String, AttributeValue> =
            std::collections::HashMap::new();
        expected_values.insert(":value0".to_owned(), "S".into_attr());
        assert_eq!(
            filter_expression,
            "attribute_type(#name, :value0)".to_owned(),
        );
        assert_eq!(attribute_names, expected_names);
        assert_eq!(attribute_values, expected_values);
    }

    #[test]
    fn test_contains_filter_expression() {
        reset_value_id();

        let cond = User::filter_expression(User::name()).contains("boku");
        let (filter_expression, attribute_names, attribute_values) = cond.build();
        let mut expected_names: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        expected_names.insert("#name".to_owned(), "name".to_owned());
        let mut expected_values: std::collections::HashMap<String, AttributeValue> =
            std::collections::HashMap::new();
        expected_values.insert(":value0".to_owned(), "boku".into_attr());
        assert_eq!(filter_expression, "contains(#name, :value0)".to_owned(),);
        assert_eq!(attribute_names, expected_names);
        assert_eq!(attribute_values, expected_values);
    }
}
