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
}
