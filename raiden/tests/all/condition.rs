#[cfg(test)]
mod tests {

    #[cfg(test)]
    use pretty_assertions::assert_eq;
    use raiden::condition::*;
    use raiden::*;

    #[derive(Raiden)]
    #[raiden(table_name = "user")]
    #[derive(Debug, Clone)]
    pub struct User {
        #[raiden(partition_key)]
        id: String,
        name: String,
    }

    #[test]
    fn test_attribute_exists_condition() {
        let cond = User::condition().attr_exists(User::name());
        let (condition_expression, attribute_names, _attribute_values) = cond.build();
        let mut expected_names: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        expected_names.insert("#name".to_owned(), "name".to_owned());
        assert_eq!(condition_expression, "attribute_exists(#name)".to_owned(),);
        assert_eq!(attribute_names, expected_names);
    }

    #[test]
    fn test_not_attribute_exists_condition() {
        let cond = User::condition().not().attr_exists(User::name());
        let (condition_expression, attribute_names, _attribute_values) = cond.build();
        let mut expected_names: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        expected_names.insert("#name".to_owned(), "name".to_owned());
        assert_eq!(
            condition_expression,
            "NOT (attribute_exists(#name))".to_owned(),
        );
        assert_eq!(attribute_names, expected_names);
    }

    #[test]
    fn test_attribute_not_exists_condition() {
        let cond = User::condition().attr_not_exists(User::name());
        let (condition_expression, attribute_names, _attribute_values) = cond.build();
        let mut expected_names: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        expected_names.insert("#name".to_owned(), "name".to_owned());
        assert_eq!(
            condition_expression,
            "attribute_not_exists(#name)".to_owned(),
        );
        assert_eq!(attribute_names, expected_names);
    }

    #[test]
    fn test_attribute_not_exists_condition_and_conjunction() {
        let cond = User::condition()
            .attr_not_exists(User::name())
            .and(User::condition().attr_not_exists(User::id()));
        let (condition_expression, attribute_names, _attribute_values) = cond.build();
        let mut expected_names: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        expected_names.insert("#name".to_owned(), "name".to_owned());
        expected_names.insert("#id".to_owned(), "id".to_owned());
        assert_eq!(
            condition_expression,
            "attribute_not_exists(#name) AND (attribute_not_exists(#id))".to_owned()
        );
        assert_eq!(attribute_names, expected_names);
    }

    #[test]
    fn test_attribute_type_condition() {
        let cond = User::condition().attr_type(User::id(), raiden::AttributeType::S);
        let (condition_expression, attribute_names, attribute_values) = cond.build();
        let mut expected_names: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        expected_names.insert("#id".to_owned(), "id".to_owned());
        let mut expected_values: raiden::AttributeValues = std::collections::HashMap::new();
        expected_values.insert(
            ":typeS".to_owned(),
            raiden::AttributeValue {
                s: Some("S".to_string()),
                ..raiden::AttributeValue::default()
            },
        );

        assert_eq!(
            condition_expression,
            "attribute_type(#id, :typeS)".to_owned()
        );
        assert_eq!(attribute_names, expected_names);
        assert_eq!(attribute_values, expected_values);
    }

    #[test]
    fn test_begins_with_condition() {
        let cond = User::condition().begins_with(User::name(), "boku");
        let (condition_expression, attribute_names, attribute_values) = cond.build();
        let mut expected_names: raiden::AttributeNames = std::collections::HashMap::new();
        expected_names.insert("#name".to_owned(), "name".to_owned());
        let mut expected_values: raiden::AttributeValues = std::collections::HashMap::new();
        expected_values.insert(
            ":begins_with_17d8e2e8233d9a6ae428061cb2cdf226".to_owned(),
            raiden::AttributeValue {
                s: Some("boku".to_string()),
                ..raiden::AttributeValue::default()
            },
        );

        assert_eq!(
            condition_expression,
            "begins_with(#name, :begins_with_17d8e2e8233d9a6ae428061cb2cdf226)".to_owned()
        );
        assert_eq!(attribute_names, expected_names);
        assert_eq!(attribute_values, expected_values);
    }

    #[test]
    fn test_and_condition() {
        let cond = User::condition()
            .attr_exists(User::name())
            .and(User::condition().attr_exists(User::id()));
        let (condition_expression, attribute_names, _attribute_values) = cond.build();
        let mut expected_names: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        expected_names.insert("#id".to_owned(), "id".to_owned());
        expected_names.insert("#name".to_owned(), "name".to_owned());
        assert_eq!(
            condition_expression,
            "attribute_exists(#name) AND (attribute_exists(#id))".to_owned(),
        );
        assert_eq!(attribute_names, expected_names);
    }

    #[test]
    fn test_three_and_condition() {
        let cond = User::condition().attr_exists(User::name()).and(
            User::condition().attr_exists(User::id()).and(
                User::condition()
                    .attr_exists(User::id())
                    .and(User::condition().attr_exists(User::id())),
            ),
        );
        let (condition_expression, attribute_names, _attribute_values) = cond.build();
        let mut expected_names: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        expected_names.insert("#id".to_owned(), "id".to_owned());
        expected_names.insert("#name".to_owned(), "name".to_owned());
        assert_eq!(
            condition_expression,
            "attribute_exists(#name) AND (attribute_exists(#id) AND (attribute_exists(#id) AND (attribute_exists(#id))))".to_owned(),
        );
        assert_eq!(attribute_names, expected_names);
    }

    #[test]
    fn test_or_condition() {
        let cond = User::condition()
            .attr_exists(User::name())
            .or(User::condition().attr_exists(User::id()));
        let (condition_expression, attribute_names, _attribute_values) = cond.build();
        let mut expected_names: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        expected_names.insert("#id".to_owned(), "id".to_owned());
        expected_names.insert("#name".to_owned(), "name".to_owned());
        assert_eq!(
            condition_expression,
            "attribute_exists(#name) OR (attribute_exists(#id))".to_owned(),
        );
        assert_eq!(attribute_names, expected_names);
    }

    #[test]
    fn test_three_or_condition() {
        let cond = User::condition()
            .attr_exists(User::name())
            .or(User::condition()
                .attr_exists(User::id())
                .or(User::condition()
                    .attr_exists(User::id())
                    .or(User::condition().attr_exists(User::id()))));
        let (condition_expression, attribute_names, _attribute_values) = cond.build();
        let mut expected_names: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        expected_names.insert("#id".to_owned(), "id".to_owned());
        expected_names.insert("#name".to_owned(), "name".to_owned());
        assert_eq!(
            condition_expression,
            "attribute_exists(#name) OR (attribute_exists(#id) OR (attribute_exists(#id) OR (attribute_exists(#id))))".to_owned(),
        );
        assert_eq!(attribute_names, expected_names);
    }

    #[test]
    fn test_cmp_eq_attr_attr_condition() {
        let cond = User::condition().attr(User::name()).eq_attr(User::name());
        let (condition_expression, attribute_names, _attribute_values) = cond.build();
        let mut expected_names: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        expected_names.insert("#name".to_owned(), "name".to_owned());
        assert_eq!(condition_expression, "#name = #name".to_owned(),);
        assert_eq!(attribute_names, expected_names);
    }

    #[test]
    fn test_cmp_eq_value_attr_condition() {
        reset_value_id();
        let cond = User::condition().value("bokuweb").eq_attr(User::name());
        let (condition_expression, attribute_names, attribute_values) = cond.build();
        let mut expected_names: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        expected_names.insert("#name".to_owned(), "name".to_owned());
        let mut expected_values: raiden::AttributeValues = std::collections::HashMap::new();
        expected_values.insert(
            ":value0".to_owned(),
            raiden::AttributeValue {
                s: Some("bokuweb".to_string()),
                ..raiden::AttributeValue::default()
            },
        );
        assert_eq!(condition_expression, ":value0 = #name".to_owned(),);
        assert_eq!(attribute_names, expected_names);
        assert_eq!(attribute_values, expected_values);
    }
}
