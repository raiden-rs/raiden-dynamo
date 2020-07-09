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
        removable: bool,
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

    #[test]
    fn test_delete_item_with_condition() {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = DeleteTest0::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let cond = DeleteTest0::condition()
                .attr(DeleteTest0::removable())
                .eq_value(true);
            let res = client.delete("id0").condition(cond.clone()).run().await;
            assert_eq!(res.is_ok(), false);
            let res = client.delete("id1").condition(cond).run().await;
            assert_eq!(res.is_ok(), true);
        }
        rt.block_on(example());
    }

    #[derive(Raiden, Debug, Clone)]
    pub struct DeleteTest1 {
        #[raiden(partition_key)]
        id: String,
        name: String,
        #[raiden(sort_key)]
        year: usize,
    }

    #[test]
    fn test_delete_item_with_sort_key() {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let client = DeleteTest1::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });

            let res = client.delete("id0", 1999).run().await;
            assert_eq!(res.is_ok(), true);
        }
        rt.block_on(example());
    }
}
