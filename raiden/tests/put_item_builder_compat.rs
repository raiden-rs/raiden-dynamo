//! Source compatibility of `put_item_builder()` and `raiden::Builder` with
//! 0.0.94 and earlier. These tests only build values; they do not talk to
//! DynamoDB.

use raiden::*;

#[derive(Raiden, Debug, Clone, PartialEq)]
#[raiden(table_name = "user")]
pub struct Profile {
    #[raiden(partition_key)]
    id: String,
    name: String,
    nickname: Option<String>,
    age: Option<u32>,
}

#[derive(Raiden, Debug, Clone, PartialEq)]
#[raiden(table_name = "user")]
pub struct ProfileWithUuid {
    #[raiden(partition_key)]
    #[raiden(uuid)]
    id: String,
    name: String,
    nickname: Option<String>,
}

fn expected(nickname: Option<&str>, age: Option<u32>) -> ProfilePutItemInput {
    ProfilePutItemInput {
        id: "id0".to_owned(),
        name: "bokuweb".to_owned(),
        nickname: nickname.map(str::to_owned),
        age,
    }
}

#[test]
fn optional_setters_keep_the_builder_type() {
    for (nickname, age) in [
        (None, None),
        (Some("boku"), None),
        (None, Some(20)),
        (Some("boku"), Some(20)),
    ] {
        // Setting optional fields conditionally does not change the builder
        // type, so the builder can be reassigned in a branch.
        let mut builder = Profile::put_item_builder()
            .id("id0".to_owned())
            .name("bokuweb".to_owned());
        if let Some(nickname) = nickname {
            builder = builder.nickname(nickname.to_owned());
        }
        if let Some(age) = age {
            builder = builder.age(age);
        }
        assert_eq!(builder.build(), expected(nickname, age));
    }
}

#[test]
fn with_option_setters_accept_none_and_some() {
    let input = Profile::put_item_builder()
        .nickname_with_option(None)
        .age_with_option(Some(3))
        .name("bokuweb".to_owned())
        .id("id0".to_owned())
        .build();
    assert_eq!(input, expected(None, Some(3)));
}

#[test]
fn optional_setters_can_come_before_required_ones() {
    let mut builder = Profile::put_item_builder();
    builder = builder.nickname("boku".to_owned());
    let input = builder
        .id("id0".to_owned())
        .name("bokuweb".to_owned())
        .build();
    assert_eq!(input, expected(Some("boku"), None));
}

// `Default<Struct>PutItemInputBuilder` names the builder returned by
// `put_item_builder()` so it can appear in signatures.
fn start_profile() -> DefaultProfilePutItemInputBuilder {
    Profile::put_item_builder()
}

#[test]
fn default_builder_type_alias_is_available() {
    let input = start_profile()
        .id("id0".to_owned())
        .name("bokuweb".to_owned())
        .build();
    assert_eq!(input, expected(None, None));

    let with_uuid: DefaultProfileWithUuidPutItemInputBuilder = ProfileWithUuid::put_item_builder();
    let input = with_uuid.name("bokuweb".to_owned()).build();
    assert_eq!(
        input,
        ProfileWithUuidPutItemInput {
            name: "bokuweb".to_owned(),
            nickname: None,
        }
    );
}

// `raiden::Builder` is the safe-builder derive again. It accepts (and
// ignores) `#[builder(...)]` attributes other than `method_name`.
#[derive(Clone, Debug, PartialEq, Builder)]
#[builder(setter(into))]
pub struct Bookmark {
    pub id: String,
    pub note: Option<String>,
}

#[test]
fn raiden_builder_is_the_safe_builder_derive() {
    let mut builder = Bookmark::builder().id("b0".to_owned());
    builder = builder.note("memo".to_owned());
    assert_eq!(
        builder.build(),
        Bookmark {
            id: "b0".to_owned(),
            note: Some("memo".to_owned()),
        }
    );
    let _: DefaultBookmarkBuilder = Bookmark::builder();
}

// bon's derive stays reachable for code written against 0.0.95 to 0.0.97.
#[derive(Clone, Debug, PartialEq, BonBuilder)]
#[builder(crate = ::raiden::bon)]
pub struct BonItem {
    pub id: String,
    pub note: Option<String>,
}

#[test]
fn bon_builder_is_available_under_a_distinct_name() {
    let item = BonItem::builder()
        .id("x".to_owned())
        .maybe_note(None)
        .build();
    assert_eq!(
        item,
        BonItem {
            id: "x".to_owned(),
            note: None,
        }
    );
}
