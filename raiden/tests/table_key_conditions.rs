use raiden::*;

#[derive(Raiden)]
#[raiden(lsi(name = "nameIndex", sort_key = "name"))]
#[allow(dead_code)]
struct User {
    #[raiden(partition_key)]
    id: String,
    #[raiden(sort_key)]
    #[raiden(omit_lsi = "nameIndex")]
    created_at: String,
    name: String,
}

fn client() -> UserClient {
    unimplemented!()
}

#[test]
fn typed_and_legacy_conditions_are_accepted_by_table_query() {
    if false {
        let typed = User::partition_key_condition()
            .eq("id")
            .and(User::sort_key_condition().begins_with("2026"));
        let _ = client().query().key_condition(typed);

        let legacy = User::key_condition(User::id()).eq("id");
        let _ = client().query().key_condition(legacy);

        let legacy_non_key = User::key_condition(User::name()).eq("name");
        let _ = client().query().key_condition(legacy_non_key);

        let legacy_explicit = User::key_condition(User::id()).eq("id");
        let _ = client()
            .query()
            .key_condition::<UserKeyConditionToken>(legacy_explicit);

        let partition_only = User::partition_key_condition().eq("id");
        let _ = client().query().key_condition(partition_only);

        let projected = User::partition_key_condition().eq("id");
        let _ = client()
            .query()
            .project::<UserNameIndexItem>()
            .key_condition(projected);
    }
}
