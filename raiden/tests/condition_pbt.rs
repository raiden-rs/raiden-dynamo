use std::collections::{HashMap, HashSet};
use std::fmt::Write;

use proptest::prelude::*;
use raiden::condition::ConditionBuilder;
use raiden::*;

#[allow(dead_code)]
#[derive(Clone, Raiden)]
#[raiden(table_name = "condition_pbt")]
struct Item {
    #[raiden(partition_key)]
    id: String,
    score: i32,
    metadata: HashMap<String, i32>,
}

fn value_placeholders(expression: &str) -> Vec<&str> {
    expression
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == ':'))
        .filter(|part| part.starts_with(":value"))
        .collect()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn nested_conditions_preserve_every_binding(
        lower in any::<i32>(),
        upper in any::<i32>(),
        candidates in proptest::collection::vec(any::<i32>(), 1..101),
        excluded in any::<i32>(),
    ) {
        let condition = Item::condition()
            .between(Item::score(), lower, upper)
            .or(Item::condition().in_values(Item::score(), candidates.clone()))
            .not()
            .and(Item::condition().attr(Item::score()).ne_value(excluded));
        let (expression, names, bindings) = condition.build();
        let placeholders = value_placeholders(&expression);

        prop_assert_eq!(placeholders.len(), candidates.len() + 3);
        prop_assert_eq!(bindings.len(), placeholders.len());
        prop_assert_eq!(
            placeholders.iter().copied().collect::<HashSet<_>>().len(),
            placeholders.len()
        );
        prop_assert_eq!(names, HashMap::from([("#score".into(), "score".into())]));
        prop_assert_eq!(
            expression.as_str(),
            format!(
                "(NOT (#score BETWEEN {} AND {} OR (#score IN ({})))) AND (#score <> {})",
                placeholders[0],
                placeholders[1],
                placeholders[2..placeholders.len() - 1].join(", "),
                placeholders[placeholders.len() - 1],
            )
        );

        let expected = [lower, upper]
            .into_iter()
            .chain(candidates)
            .chain([excluded]);
        for (placeholder, value) in placeholders.into_iter().zip(expected) {
            let expected_value = value.into_attr();
            prop_assert_eq!(bindings.get(placeholder), Some(&expected_value));
        }
    }

    #[test]
    fn dynamic_map_keys_do_not_share_placeholders(suffix in "[a-z0-9_-]{1,12}") {
        let punctuated_key = format!("k.{suffix}");
        let mut formerly_colliding_key = String::from("path_");
        for byte in punctuated_key.as_bytes() {
            write!(&mut formerly_colliding_key, "{byte:02x}").unwrap();
        }

        let condition = Item::condition()
            .attr_exists(Item::metadata().key(punctuated_key.clone()))
            .and(Item::condition().attr_exists(Item::metadata().key(formerly_colliding_key.clone())));
        let (expression, names, values) = condition.build();
        let aliases: Vec<_> = expression
            .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '#' || ch == '_'))
            .filter(|part| part.starts_with('#'))
            .collect();

        prop_assert_eq!(aliases.len(), 4);
        prop_assert_eq!(aliases[0], "#metadata");
        prop_assert_eq!(aliases[2], "#metadata");
        prop_assert_ne!(aliases[1], aliases[3]);
        prop_assert_eq!(names.len(), 3);
        prop_assert_eq!(names.get(aliases[1]), Some(&punctuated_key));
        prop_assert_eq!(names.get(aliases[3]), Some(&formerly_colliding_key));
        prop_assert!(values.is_empty());
    }
}
