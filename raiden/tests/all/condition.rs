#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    #[cfg(test)]
    use pretty_assertions::assert_eq;
    use raiden::condition::*;
    use raiden::*;

    #[allow(dead_code)]
    #[derive(Raiden)]
    #[raiden(table_name = "user")]
    #[derive(Debug, Clone)]
    pub struct User {
        #[raiden(partition_key)]
        id: String,
        name: String,
    }

    #[allow(dead_code)]
    #[derive(Raiden)]
    #[raiden(table_name = "organization")]
    pub struct Organization {
        #[raiden(partition_key)]
        id: String,
        admin_ids: Vec<String>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, RaidenDocument)]
    pub struct Profile {
        level: usize,
        nickname: String,
    }

    #[derive(Raiden)]
    #[raiden(table_name = "user")]
    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    pub struct UserWithMapCondition {
        #[raiden(partition_key)]
        id: String,
        profile: Profile,
        metadata: HashMap<String, usize>,
    }

    #[test]
    fn test_attribute_exists_condition() {
        let cond = User::condition().attr_exists(User::name());
        let (condition_expression, attribute_names, _attribute_values) = cond.build();
        let mut expected_names: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        expected_names.insert("#name".to_owned(), "name".to_owned());
        assert_eq!(condition_expression, "attribute_exists(#name)".to_owned());
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
    fn test_attribute_not_exists_condition_and_operator() {
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
        let attr_value: raiden::AttributeValue = "S".into_attr();
        expected_values.insert(":typeS".to_owned(), attr_value);

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
        let attr_value: raiden::AttributeValue = "boku".into_attr();
        expected_values.insert(
            ":begins_with_17d8e2e8233d9a6ae428061cb2cdf226".to_owned(),
            attr_value,
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
        assert_eq!(condition_expression, "#name = #name".to_owned());
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
        let attr_value: raiden::AttributeValue = "bokuweb".into_attr();
        expected_values.insert(":value0".to_owned(), attr_value);

        assert_eq!(condition_expression, ":value0 = #name".to_owned());
        assert_eq!(attribute_names, expected_names);
        assert_eq!(attribute_values, expected_values);
    }

    #[test]
    fn test_contains_size_and_logical_condition() {
        reset_value_id();
        let preserve_last_admin = Organization::condition()
            .not()
            .contains(Organization::admin_ids(), "member#1")
            .or(Organization::condition()
                .size(Organization::admin_ids())
                .ge(2_usize));
        let cond = Organization::condition()
            .attr_exists(Organization::id())
            .and(preserve_last_admin);

        let (condition_expression, attribute_names, attribute_values) = cond.build();

        assert_eq!(
            condition_expression,
            "attribute_exists(#id) AND (NOT (contains(#admin_ids, :value0)) OR (size(#admin_ids) >= :value1))"
        );
        assert_eq!(
            attribute_names,
            std::collections::HashMap::from([
                ("#id".to_owned(), "id".to_owned()),
                ("#admin_ids".to_owned(), "admin_ids".to_owned()),
            ])
        );
        assert_eq!(
            attribute_values,
            std::collections::HashMap::from([
                (":value0".to_owned(), "member#1".into_attr()),
                (":value1".to_owned(), 2_usize.into_attr()),
            ])
        );
    }

    #[test]
    fn test_size_numeric_comparison_operators() {
        let cases = [
            ("=", 0_usize),
            ("<>", 1),
            ("<", 2),
            ("<=", 3),
            (">", 4),
            (">=", 5),
        ];

        reset_value_id();
        let conditions = [
            Organization::condition()
                .size(Organization::admin_ids())
                .eq(2_usize)
                .build(),
            Organization::condition()
                .size(Organization::admin_ids())
                .ne(2_usize)
                .build(),
            Organization::condition()
                .size(Organization::admin_ids())
                .lt(2_usize)
                .build(),
            Organization::condition()
                .size(Organization::admin_ids())
                .le(2_usize)
                .build(),
            Organization::condition()
                .size(Organization::admin_ids())
                .gt(2_usize)
                .build(),
            Organization::condition()
                .size(Organization::admin_ids())
                .ge(2_usize)
                .build(),
        ];

        for ((operator, id), (expression, names, values)) in cases.into_iter().zip(conditions) {
            let placeholder = format!(":value{id}");
            assert_eq!(
                expression,
                format!("size(#admin_ids) {operator} {placeholder}")
            );
            assert_eq!(
                names,
                std::collections::HashMap::from([(
                    "#admin_ids".to_owned(),
                    "admin_ids".to_owned()
                )])
            );
            assert_eq!(
                values,
                std::collections::HashMap::from([(placeholder, 2_usize.into_attr())])
            );
        }
    }

    #[test]
    fn test_map_key_attribute_exists_condition() {
        let cond = UserWithMapCondition::condition()
            .attr_exists(UserWithMapCondition::metadata().key("score"));
        let (condition_expression, attribute_names, _attribute_values) = cond.build();

        let mut expected_names = std::collections::HashMap::new();
        expected_names.insert("#metadata".to_owned(), "metadata".to_owned());
        expected_names.insert("#score".to_owned(), "score".to_owned());

        assert_eq!(
            condition_expression,
            "attribute_exists(#metadata.#score)".to_owned()
        );
        assert_eq!(attribute_names, expected_names);
    }

    #[test]
    fn test_document_field_eq_value_condition() {
        reset_value_id();
        let cond = UserWithMapCondition::condition()
            .attr(UserWithMapCondition::profile().field(Profile::level()))
            .eq_value(3);
        let (condition_expression, attribute_names, attribute_values) = cond.build();

        let mut expected_names = std::collections::HashMap::new();
        expected_names.insert("#profile".to_owned(), "profile".to_owned());
        expected_names.insert("#level".to_owned(), "level".to_owned());

        let placeholder = attribute_values.keys().next().cloned().unwrap();
        let mut expected_values = std::collections::HashMap::new();
        expected_values.insert(placeholder.clone(), 3.into_attr());

        assert_eq!(
            condition_expression,
            format!("#profile.#level = {placeholder}")
        );
        assert_eq!(attribute_names, expected_names);
        assert_eq!(attribute_values, expected_values);
    }
}
