#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashMap};

    use pretty_assertions::assert_eq;
    use raiden::*;

    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    pub struct Profile {
        display_name: String,
        level: usize,
    }

    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, RaidenDocument)]
    pub struct DerivedProfile {
        display_name: String,
        level: usize,
    }

    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, RaidenDocument)]
    #[serde(tag = "type")]
    pub enum Message {
        Request {
            id: String,
            method: String,
            params: HashMap<String, usize>,
        },
        Response {
            id: String,
            result: String,
        },
    }

    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, RaidenDocument)]
    #[serde(tag = "kind", rename_all = "snake_case")]
    pub enum RenamedMessage {
        SendRequest { id: String },
        SendResponse { id: String },
    }

    #[derive(Raiden)]
    #[raiden(table_name = "user")]
    #[derive(Debug, Clone, PartialEq)]
    pub struct UserWithDocument {
        #[raiden(partition_key)]
        id: String,
        metadata: HashMap<String, usize>,
        flags: BTreeMap<String, bool>,
        profiles: HashMap<String, Document<Profile>>,
        profile: Document<Profile>,
    }

    #[derive(Raiden)]
    #[raiden(table_name = "user")]
    #[derive(Debug, Clone, PartialEq)]
    pub struct UserWithDerivedDocument {
        #[raiden(partition_key)]
        id: String,
        profile: DerivedProfile,
        profiles: HashMap<String, DerivedProfile>,
    }

    #[derive(Raiden)]
    #[raiden(table_name = "user")]
    #[derive(Debug, Clone, PartialEq)]
    pub struct UserWithEnumDocument {
        #[raiden(partition_key)]
        id: String,
        message: Message,
    }

    fn sample_profile() -> Profile {
        Profile {
            display_name: "bokuweb".to_owned(),
            level: 3,
        }
    }

    fn sample_derived_profile() -> DerivedProfile {
        DerivedProfile {
            display_name: "bokuweb".to_owned(),
            level: 3,
        }
    }

    fn sample_metadata() -> HashMap<String, usize> {
        let mut metadata = HashMap::new();
        metadata.insert("score".to_owned(), 42);
        metadata.insert("visits".to_owned(), 7);
        metadata
    }

    fn sample_flags() -> BTreeMap<String, bool> {
        let mut flags = BTreeMap::new();
        flags.insert("active".to_owned(), true);
        flags.insert("staff".to_owned(), false);
        flags
    }

    fn sample_profiles() -> HashMap<String, Document<Profile>> {
        let mut profiles = HashMap::new();
        profiles.insert(
            "primary".to_owned(),
            Document::new(Profile {
                display_name: "raiden".to_owned(),
                level: 10,
            }),
        );
        profiles
    }

    fn sample_derived_profiles() -> HashMap<String, DerivedProfile> {
        let mut profiles = HashMap::new();
        profiles.insert(
            "primary".to_owned(),
            DerivedProfile {
                display_name: "raiden".to_owned(),
                level: 10,
            },
        );
        profiles
    }

    fn sample_message() -> Message {
        let mut params = HashMap::new();
        params.insert("attempt".to_owned(), 3);

        Message::Request {
            id: "msg#1".to_owned(),
            method: "send".to_owned(),
            params,
        }
    }

    fn sample_response_message() -> Message {
        Message::Response {
            id: "msg#2".to_owned(),
            result: "ok".to_owned(),
        }
    }

    #[test]
    fn test_from_item_with_document_and_maps() {
        let metadata = sample_metadata();
        let flags = sample_flags();
        let profiles = sample_profiles();
        let profile = sample_profile();

        let mut item = HashMap::new();
        item.insert("id".to_owned(), "user#1".to_owned().into_attr());
        item.insert("metadata".to_owned(), metadata.clone().into_attr());
        item.insert("flags".to_owned(), flags.clone().into_attr());
        item.insert("profiles".to_owned(), profiles.clone().into_attr());
        item.insert(
            "profile".to_owned(),
            Document::new(profile.clone()).into_attr(),
        );

        assert_eq!(
            UserWithDocument::from_item(item).unwrap(),
            UserWithDocument {
                id: "user#1".to_owned(),
                metadata,
                flags,
                profiles,
                profile: Document::new(profile),
            }
        );
    }

    #[test]
    fn test_from_item_with_raiden_document() {
        let profile = sample_derived_profile();
        let profiles = sample_derived_profiles();

        let mut item = HashMap::new();
        item.insert("id".to_owned(), "user#derived".to_owned().into_attr());
        item.insert("profile".to_owned(), profile.clone().into_attr());
        item.insert("profiles".to_owned(), profiles.clone().into_attr());

        assert_eq!(
            UserWithDerivedDocument::from_item(item).unwrap(),
            UserWithDerivedDocument {
                id: "user#derived".to_owned(),
                profile,
                profiles,
            }
        );
    }

    #[test]
    fn test_from_item_with_raiden_document_enum() {
        let message = sample_message();

        let mut item = HashMap::new();
        item.insert("id".to_owned(), "user#message".to_owned().into_attr());
        item.insert("message".to_owned(), message.clone().into_attr());

        assert_eq!(
            UserWithEnumDocument::from_item(item).unwrap(),
            UserWithEnumDocument {
                id: "user#message".to_owned(),
                message,
            }
        );
    }

    #[test]
    fn test_from_item_with_raiden_document_enum_response_variant() {
        let message = sample_response_message();

        let mut item = HashMap::new();
        item.insert("id".to_owned(), "user#response".to_owned().into_attr());
        item.insert("message".to_owned(), message.clone().into_attr());

        assert_eq!(
            UserWithEnumDocument::from_item(item).unwrap(),
            UserWithEnumDocument {
                id: "user#response".to_owned(),
                message,
            }
        );
    }

    #[test]
    fn test_from_item_with_raiden_document_enum_empty_map_payload() {
        let message = Message::Request {
            id: "msg#empty".to_owned(),
            method: "send".to_owned(),
            params: HashMap::new(),
        };

        let mut item = HashMap::new();
        item.insert("id".to_owned(), "user#empty-message".to_owned().into_attr());
        item.insert("message".to_owned(), message.clone().into_attr());

        assert_eq!(
            UserWithEnumDocument::from_item(item).unwrap(),
            UserWithEnumDocument {
                id: "user#empty-message".to_owned(),
                message,
            }
        );
    }

    #[test]
    fn test_raiden_document_enum_from_attribute_round_trip() {
        for message in [sample_message(), sample_response_message()] {
            let attr = message.clone().into_attr();
            let decoded = Message::from_attr(Some(attr)).unwrap();

            assert_eq!(decoded, message);
        }
    }

    #[test]
    fn test_document_wrapper_with_tagged_enum_round_trip() {
        for message in [sample_message(), sample_response_message()] {
            let document = Document::new(message);
            let attr = document.clone().into_attr();
            let decoded = Document::<Message>::from_attr(Some(attr)).unwrap();

            assert_eq!(decoded, document);
        }
    }

    #[test]
    fn test_raiden_document_enum_attribute_names_are_unprojected() {
        assert_eq!(Message::attribute_names(), None);
        assert_eq!(Message::projection_expression(), None);
    }

    #[test]
    fn test_raiden_document_enum_from_item() {
        let mut params = HashMap::new();
        params.insert("attempt".to_owned(), 3);

        let mut item = HashMap::new();
        item.insert("type".to_owned(), "Request".to_owned().into_attr());
        item.insert("id".to_owned(), "msg#1".to_owned().into_attr());
        item.insert("method".to_owned(), "send".to_owned().into_attr());
        item.insert("params".to_owned(), params.clone().into_attr());

        assert_eq!(
            Message::from_item(item).unwrap(),
            Message::Request {
                id: "msg#1".to_owned(),
                method: "send".to_owned(),
                params,
            }
        );
    }

    #[test]
    fn test_raiden_document_enum_from_item_preserves_serde_variant_rename_all() {
        let mut item = HashMap::new();
        item.insert("kind".to_owned(), "send_request".to_owned().into_attr());
        item.insert("id".to_owned(), "msg#renamed".to_owned().into_attr());

        assert_eq!(
            RenamedMessage::from_item(item).unwrap(),
            RenamedMessage::SendRequest {
                id: "msg#renamed".to_owned(),
            }
        );
    }

    #[test]
    fn test_raiden_document_enum_from_item_response_variant() {
        let mut item = HashMap::new();
        item.insert("type".to_owned(), "Response".to_owned().into_attr());
        item.insert("id".to_owned(), "msg#2".to_owned().into_attr());
        item.insert("result".to_owned(), "ok".to_owned().into_attr());

        assert_eq!(
            Message::from_item(item).unwrap(),
            Message::Response {
                id: "msg#2".to_owned(),
                result: "ok".to_owned(),
            }
        );
    }

    #[test]
    fn test_raiden_document_enum_from_item_rejects_unknown_tag() {
        let mut item = HashMap::new();
        item.insert("type".to_owned(), "Unknown".to_owned().into_attr());
        item.insert("id".to_owned(), "msg#unknown".to_owned().into_attr());

        assert!(Message::from_item(item).is_err());
    }

    #[test]
    fn test_raiden_document_enum_from_item_rejects_missing_tag() {
        let mut item = HashMap::new();
        item.insert("id".to_owned(), "msg#missing-tag".to_owned().into_attr());
        item.insert("method".to_owned(), "send".to_owned().into_attr());
        item.insert(
            "params".to_owned(),
            HashMap::<String, usize>::new().into_attr(),
        );

        assert!(Message::from_item(item).is_err());
    }

    #[test]
    fn test_raiden_document_enum_from_item_rejects_missing_required_field() {
        let mut item = HashMap::new();
        item.insert("type".to_owned(), "Request".to_owned().into_attr());
        item.insert("id".to_owned(), "msg#missing-field".to_owned().into_attr());
        item.insert(
            "params".to_owned(),
            HashMap::<String, usize>::new().into_attr(),
        );

        assert!(Message::from_item(item).is_err());
    }

    #[tokio::test]
    async fn test_put_input_with_document_and_maps() {
        let client = crate::all::create_client_from_struct!(UserWithDocument);

        let metadata = sample_metadata();
        let flags = sample_flags();
        let profiles = sample_profiles();
        let profile = sample_profile();

        let input_item = UserWithDocumentPutItemInput {
            id: "user#1".to_owned(),
            metadata: metadata.clone(),
            flags: flags.clone(),
            profiles: profiles.clone(),
            profile: Document::new(profile.clone()),
        };

        #[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
        let input = client.put(input_item).input;
        #[cfg(feature = "aws-sdk")]
        let input = client.put(input_item).builder.build().unwrap();

        #[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
        let item = input.item;
        #[cfg(feature = "aws-sdk")]
        let item = input.item.unwrap();

        assert_eq!(item.get("id"), Some(&"user#1".to_owned().into_attr()));
        assert_eq!(item.get("metadata"), Some(&metadata.into_attr()));
        assert_eq!(item.get("flags"), Some(&flags.into_attr()));
        assert_eq!(item.get("profiles"), Some(&profiles.into_attr()));
        assert_eq!(
            item.get("profile"),
            Some(&Document::new(profile).into_attr())
        );
    }

    #[tokio::test]
    async fn test_put_input_with_empty_maps() {
        let client = crate::all::create_client_from_struct!(UserWithDocument);
        let profile = sample_profile();

        let input_item = UserWithDocumentPutItemInput {
            id: "user#empty".to_owned(),
            metadata: HashMap::new(),
            flags: BTreeMap::new(),
            profiles: HashMap::new(),
            profile: Document::new(profile.clone()),
        };

        #[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
        let input = client.put(input_item).input;
        #[cfg(feature = "aws-sdk")]
        let input = client.put(input_item).builder.build().unwrap();

        #[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
        let item = input.item;
        #[cfg(feature = "aws-sdk")]
        let item = input.item.unwrap();

        assert_eq!(item.get("id"), Some(&"user#empty".to_owned().into_attr()));
        assert_eq!(
            item.get("metadata"),
            Some(&HashMap::<String, usize>::new().into_attr())
        );
        assert_eq!(
            item.get("flags"),
            Some(&BTreeMap::<String, bool>::new().into_attr())
        );
        assert_eq!(
            item.get("profiles"),
            Some(&HashMap::<String, Document<Profile>>::new().into_attr())
        );
        assert_eq!(
            item.get("profile"),
            Some(&Document::new(profile).into_attr())
        );
    }

    #[tokio::test]
    async fn test_put_input_with_raiden_document() {
        let client = crate::all::create_client_from_struct!(UserWithDerivedDocument);
        let profile = sample_derived_profile();
        let profiles = sample_derived_profiles();

        let input_item = UserWithDerivedDocumentPutItemInput {
            id: "user#derived".to_owned(),
            profile: profile.clone(),
            profiles: profiles.clone(),
        };

        #[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
        let input = client.put(input_item).input;
        #[cfg(feature = "aws-sdk")]
        let input = client.put(input_item).builder.build().unwrap();

        #[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
        let item = input.item;
        #[cfg(feature = "aws-sdk")]
        let item = input.item.unwrap();

        assert_eq!(item.get("id"), Some(&"user#derived".to_owned().into_attr()));
        assert_eq!(item.get("profile"), Some(&profile.into_attr()));
        assert_eq!(item.get("profiles"), Some(&profiles.into_attr()));
    }

    #[tokio::test]
    async fn test_put_input_with_raiden_document_enum() {
        let client = crate::all::create_client_from_struct!(UserWithEnumDocument);
        let message = sample_message();

        let input_item = UserWithEnumDocumentPutItemInput {
            id: "user#message".to_owned(),
            message: message.clone(),
        };

        #[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
        let input = client.put(input_item).input;
        #[cfg(feature = "aws-sdk")]
        let input = client.put(input_item).builder.build().unwrap();

        #[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
        let item = input.item;
        #[cfg(feature = "aws-sdk")]
        let item = input.item.unwrap();

        assert_eq!(item.get("id"), Some(&"user#message".to_owned().into_attr()));
        assert_eq!(item.get("message"), Some(&message.into_attr()));
    }
}
