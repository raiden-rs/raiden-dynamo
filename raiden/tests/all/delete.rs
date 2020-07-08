#[cfg(test)]
mod tests {

    #[cfg(test)]
    use pretty_assertions::assert_eq;
    use raiden::*;

    #[derive(Raiden, Debug, Clone)]
    pub struct DeleteTest0 {
        #[raiden(partition_key)]
        id: String,
        name: String,
    }

    #[test]
    fn test_delete_item() {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = DeleteTest0::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });

            let res = client.delete("id0").run().await;
            assert_eq!(res.is_ok(), true);
        }
        rt.block_on(example());
    }

    #[test]
    fn test_delete_item_with_unstored_key() {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = DeleteTest0::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });

            let res = client.delete("unstored").run().await;
            assert_eq!(res.is_ok(), true);
        }
        rt.block_on(example());
    }
}
