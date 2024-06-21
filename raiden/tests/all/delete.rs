#[cfg(test)]
mod tests {

    #[cfg(test)]
    use pretty_assertions::assert_eq;
    use raiden::*;

    #[allow(dead_code)]
    #[derive(Raiden, Debug, Clone)]
    pub struct DeleteTest0 {
        #[raiden(partition_key)]
        id: String,
        name: String,
        removable: bool,
    }

    #[tokio::test]
    async fn test_delete_item() {
        let client = crate::all::create_client_from_struct!(DeleteTest0);
        let res = client.delete("id0").run().await;

        assert_eq!(res.is_ok(), true);
    }

    #[tokio::test]
    async fn test_delete_item_with_unstored_key() {
        let client = crate::all::create_client_from_struct!(DeleteTest0);
        let res = client.delete("unstored").run().await;

        assert_eq!(res.is_ok(), true);
    }

    #[tokio::test]
    async fn test_delete_item_with_condition() {
        let client = crate::all::create_client_from_struct!(DeleteTest0);
        let cond = DeleteTest0::condition()
            .attr(DeleteTest0::removable())
            .eq_value(true);
        let res = client.delete("id0").condition(cond.clone()).run().await;
        assert_eq!(res.is_ok(), false);
        let res = client.delete("id1").condition(cond).run().await;
        assert_eq!(res.is_ok(), true);
    }

    #[allow(dead_code)]
    #[derive(Raiden, Debug, Clone)]
    pub struct DeleteTest1 {
        #[raiden(partition_key)]
        id: String,
        name: String,
        #[raiden(sort_key)]
        year: usize,
    }

    #[tokio::test]
    async fn test_delete_item_with_sort_key() {
        let client = crate::all::create_client_from_struct!(DeleteTest1);
        let res = client.delete("id0", 1999_usize).run().await;

        assert_eq!(res.is_ok(), true);
    }
}
