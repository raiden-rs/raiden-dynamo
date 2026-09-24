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
        let cond = User::condition().value("bokuweb").eq_attr(User::name());
        let (condition_expression, attribute_names, attribute_values) = cond.build();
        let mut expected_names: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        expected_names.insert("#name".to_owned(), "name".to_owned());
        let placeholder = condition_expression.strip_suffix(" = #name").unwrap();
        assert!(placeholder.starts_with(":value"));
        let mut expected_values: raiden::AttributeValues = std::collections::HashMap::new();
        let attr_value: raiden::AttributeValue = "bokuweb".into_attr();
        expected_values.insert(placeholder.to_owned(), attr_value);

        assert_eq!(condition_expression, format!("{placeholder} = #name"));
        assert_eq!(attribute_names, expected_names);
        assert_eq!(attribute_values, expected_values);
    }

    #[test]
    fn test_attribute_comparison_operators() {
        let conditions = [
            User::condition()
                .attr(User::name())
                .ne_value("boku")
                .build(),
            User::condition()
                .attr(User::name())
                .lt_value("boku")
                .build(),
            User::condition()
                .attr(User::name())
                .le_value("boku")
                .build(),
            User::condition()
                .attr(User::name())
                .gt_value("boku")
                .build(),
            User::condition()
                .attr(User::name())
                .ge_value("boku")
                .build(),
        ];
        for (operator, (expression, names, values)) in
            ["<>", "<", "<=", ">", ">="].into_iter().zip(conditions)
        {
            let placeholder = expression
                .strip_prefix(&format!("#name {operator} "))
                .expect("comparison expression has the expected operator");
            assert!(placeholder.starts_with(":value"));
            assert_eq!(names, HashMap::from([("#name".into(), "name".into())]));
            assert_eq!(
                values,
                HashMap::from([(placeholder.to_owned(), "boku".into_attr())])
            );
        }

        let attribute_conditions = [
            User::condition()
                .attr(User::id())
                .ne_attr(User::name())
                .build(),
            User::condition()
                .attr(User::id())
                .lt_attr(User::name())
                .build(),
            User::condition()
                .attr(User::id())
                .le_attr(User::name())
                .build(),
            User::condition()
                .attr(User::id())
                .gt_attr(User::name())
                .build(),
            User::condition()
                .attr(User::id())
                .ge_attr(User::name())
                .build(),
        ];
        for (operator, (expression, names, values)) in ["<>", "<", "<=", ">", ">="]
            .into_iter()
            .zip(attribute_conditions)
        {
            assert_eq!(expression, format!("#id {operator} #name"));
            assert_eq!(
                names,
                HashMap::from([("#id".into(), "id".into()), ("#name".into(), "name".into())])
            );
            assert!(values.is_empty());
        }

        let reversed_conditions = [
            User::condition()
                .value("boku")
                .ne_attr(User::name())
                .build(),
            User::condition()
                .value("boku")
                .lt_attr(User::name())
                .build(),
            User::condition()
                .value("boku")
                .le_attr(User::name())
                .build(),
            User::condition()
                .value("boku")
                .gt_attr(User::name())
                .build(),
            User::condition()
                .value("boku")
                .ge_attr(User::name())
                .build(),
        ];
        for (operator, (expression, names, values)) in ["<>", "<", "<=", ">", ">="]
            .into_iter()
            .zip(reversed_conditions)
        {
            let placeholder = expression
                .strip_suffix(&format!(" {operator} #name"))
                .expect("value is on the left of the comparison");
            assert!(placeholder.starts_with(":value"));
            assert_eq!(names, HashMap::from([("#name".into(), "name".into())]));
            assert_eq!(
                values,
                HashMap::from([(placeholder.into(), "boku".into_attr())])
            );
        }
    }

    #[test]
    fn test_between_and_in_values() {
        let condition = User::condition()
            .between(User::name(), "alpha", "omega")
            .and(User::condition().in_values(User::name(), ["alpha", "beta"]));
        let (expression, names, values) = condition.build();
        let placeholders: Vec<_> = expression
            .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == ':'))
            .filter(|part| part.starts_with(":value"))
            .collect();
        assert_eq!(placeholders.len(), 4);
        assert_eq!(
            expression,
            format!(
                "#name BETWEEN {} AND {} AND (#name IN ({}, {}))",
                placeholders[0], placeholders[1], placeholders[2], placeholders[3]
            )
        );
        assert_eq!(names, HashMap::from([("#name".into(), "name".into())]));
        assert_eq!(
            values,
            HashMap::from([
                (placeholders[0].into(), "alpha".into_attr()),
                (placeholders[1].into(), "omega".into_attr()),
                (placeholders[2].into(), "alpha".into_attr()),
                (placeholders[3].into(), "beta".into_attr()),
            ])
        );
    }

    #[test]
    fn test_between_preserves_equal_bounds_and_document_path() {
        let (expression, names, values) = UserWithMapCondition::condition()
            .between(
                UserWithMapCondition::profile().field(Profile::level()),
                7_usize,
                7_usize,
            )
            .build();
        let placeholders: Vec<_> = expression
            .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == ':'))
            .filter(|part| part.starts_with(":value"))
            .collect();
        assert_eq!(placeholders.len(), 2);
        assert_ne!(placeholders[0], placeholders[1]);
        assert_eq!(
            expression,
            format!(
                "#profile.#level BETWEEN {} AND {}",
                placeholders[0], placeholders[1]
            )
        );
        assert_eq!(
            names,
            HashMap::from([
                ("#profile".into(), "profile".into()),
                ("#level".into(), "level".into()),
            ])
        );
        assert_eq!(
            values,
            HashMap::from([
                (placeholders[0].into(), 7_usize.into_attr()),
                (placeholders[1].into(), 7_usize.into_attr()),
            ])
        );
    }

    #[test]
    fn test_in_values_accepts_one_and_100_values_in_order() {
        for count in [1_usize, 100] {
            let input: Vec<_> = (0..count).collect();
            let (expression, names, values) = UserWithMapCondition::condition()
                .in_values(
                    UserWithMapCondition::profile().field(Profile::level()),
                    input,
                )
                .build();
            let placeholders: Vec<_> = expression
                .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == ':'))
                .filter(|part| part.starts_with(":value"))
                .collect();
            assert_eq!(placeholders.len(), count);
            assert_eq!(values.len(), count);
            assert_eq!(
                expression,
                format!("#profile.#level IN ({})", placeholders.join(", "))
            );
            assert_eq!(
                names,
                HashMap::from([
                    ("#profile".into(), "profile".into()),
                    ("#level".into(), "level".into()),
                ])
            );
            for (index, placeholder) in placeholders.into_iter().enumerate() {
                assert_eq!(values.get(placeholder), Some(&index.into_attr()));
            }
        }
    }

    #[test]
    fn test_not_entire_condition_group() {
        let condition = User::condition()
            .attr_exists(User::name())
            .or(User::condition().attr_exists(User::id()))
            .not()
            .and(User::condition().attr_not_exists(User::name()));
        let (expression, names, values) = condition.build();
        assert_eq!(expression, "(NOT (attribute_exists(#name) OR (attribute_exists(#id)))) AND (attribute_not_exists(#name))");
        assert_eq!(
            names,
            HashMap::from([("#name".into(), "name".into()), ("#id".into(), "id".into()),])
        );
        assert!(values.is_empty());
    }

    #[test]
    fn test_not_of_nested_groups_keeps_all_value_bindings() {
        let condition = User::condition()
            .between(User::name(), "a", "z")
            .or(User::condition().in_values(User::name(), ["m", "n"]))
            .not()
            .and(User::condition().attr(User::id()).ne_value("blocked"));
        let (expression, names, values) = condition.build();
        let placeholders: Vec<_> = expression
            .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == ':'))
            .filter(|part| part.starts_with(":value"))
            .collect();
        assert_eq!(placeholders.len(), 5);
        assert_eq!(values.len(), 5);
        assert_eq!(
            expression,
            format!(
                "(NOT (#name BETWEEN {} AND {} OR (#name IN ({}, {})))) AND (#id <> {})",
                placeholders[0], placeholders[1], placeholders[2], placeholders[3], placeholders[4]
            )
        );
        assert_eq!(
            names,
            HashMap::from([("#name".into(), "name".into()), ("#id".into(), "id".into())])
        );
        for (placeholder, expected) in placeholders
            .into_iter()
            .zip(["a", "z", "m", "n", "blocked"])
        {
            assert_eq!(values.get(placeholder), Some(&expected.into_attr()));
        }
    }

    #[test]
    fn test_not_wraps_complete_and_group() {
        let (expression, names, values) = User::condition()
            .attr_exists(User::name())
            .and(User::condition().attr_exists(User::id()))
            .not()
            .build();
        assert_eq!(
            expression,
            "NOT (attribute_exists(#name) AND (attribute_exists(#id)))"
        );
        assert_eq!(
            names,
            HashMap::from([("#name".into(), "name".into()), ("#id".into(), "id".into())])
        );
        assert!(values.is_empty());
    }

    #[test]
    #[should_panic(expected = "IN requires between 1 and 100 values")]
    fn test_in_values_rejects_empty_list() {
        User::condition().in_values(User::name(), Vec::<String>::new());
    }

    #[test]
    #[should_panic(expected = "IN requires between 1 and 100 values")]
    fn test_in_values_rejects_more_than_100_values() {
        User::condition().in_values(User::name(), vec!["a"; 101]);
    }

    #[test]
    fn test_contains_size_and_logical_condition() {
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

        let placeholders: Vec<_> = condition_expression
            .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == ':'))
            .filter(|part| part.starts_with(":value"))
            .collect();
        assert_eq!(placeholders.len(), 2);
        assert_ne!(placeholders[0], placeholders[1]);

        assert_eq!(
            condition_expression,
            format!(
                "attribute_exists(#id) AND (NOT (contains(#admin_ids, {})) OR (size(#admin_ids) >= {}))",
                placeholders[0], placeholders[1]
            )
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
                (placeholders[0].to_owned(), "member#1".into_attr()),
                (placeholders[1].to_owned(), 2_usize.into_attr()),
            ])
        );
    }

    #[test]
    fn test_size_numeric_comparison_operators() {
        let operators = ["=", "<>", "<", "<=", ">", ">="];
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

        for (operator, (expression, names, values)) in operators.into_iter().zip(conditions) {
            let placeholder = expression
                .strip_prefix(&format!("size(#admin_ids) {operator} "))
                .unwrap();
            assert!(placeholder.starts_with(":value"));
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
                std::collections::HashMap::from([(placeholder.to_owned(), 2_usize.into_attr())])
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
    fn test_distinct_map_keys_keep_distinct_attribute_placeholders() {
        let cond = UserWithMapCondition::condition()
            .attr_exists(UserWithMapCondition::metadata().key("a-b"))
            .and(
                UserWithMapCondition::condition()
                    .attr_exists(UserWithMapCondition::metadata().key("c.d"))
                    .and(
                        UserWithMapCondition::condition()
                            .attr_exists(UserWithMapCondition::metadata().key("path_612d62")),
                    ),
            );
        let (expression, names, _) = cond.build();

        assert_eq!(
            expression,
            "attribute_exists(#metadata.#path_612d62) AND (attribute_exists(#metadata.#path_632e64) AND (attribute_exists(#metadata.#path__612d62)))"
        );
        assert_eq!(
            names,
            HashMap::from([
                ("#metadata".to_owned(), "metadata".to_owned()),
                ("#path_612d62".to_owned(), "a-b".to_owned()),
                ("#path_632e64".to_owned(), "c.d".to_owned()),
                ("#path__612d62".to_owned(), "path_612d62".to_owned()),
            ])
        );
    }

    #[test]
    fn test_document_field_eq_value_condition() {
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
