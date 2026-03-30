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

    #[derive(Raiden)]
    #[raiden(table_name = "user")]
    #[derive(Debug, Clone, PartialEq)]
    pub struct UserWithDocument {
        #[raiden(partition_key)]
        id: String,
        metadata: HashMap<String, usize>,
        flags: BTreeMap<String, bool>,
        profile: Document<Profile>,
    }

    #[test]
    fn test_from_item_with_document_and_maps() {
        let mut metadata = HashMap::new();
        metadata.insert("score".to_owned(), 42);
        metadata.insert("visits".to_owned(), 7);

        let mut flags = BTreeMap::new();
        flags.insert("active".to_owned(), true);
        flags.insert("staff".to_owned(), false);

        let profile = Profile {
            display_name: "bokuweb".to_owned(),
            level: 3,
        };

        let mut item = HashMap::new();
        item.insert("id".to_owned(), "user#1".to_owned().into_attr());
        item.insert("metadata".to_owned(), metadata.clone().into_attr());
        item.insert("flags".to_owned(), flags.clone().into_attr());
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
                profile: Document::new(profile),
            }
        );
    }

    #[tokio::test]
    async fn test_put_input_with_document_and_maps() {
        let client = crate::all::create_client_from_struct!(UserWithDocument);

        let mut metadata = HashMap::new();
        metadata.insert("score".to_owned(), 42);
        metadata.insert("visits".to_owned(), 7);

        let mut flags = BTreeMap::new();
        flags.insert("active".to_owned(), true);
        flags.insert("staff".to_owned(), false);

        let profile = Profile {
            display_name: "bokuweb".to_owned(),
            level: 3,
        };

        let input_item = UserWithDocumentPutItemInput {
            id: "user#1".to_owned(),
            metadata: metadata.clone(),
            flags: flags.clone(),
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
        assert_eq!(
            item.get("profile"),
            Some(&Document::new(profile).into_attr())
        );
    }
}
