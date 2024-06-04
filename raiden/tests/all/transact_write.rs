#[cfg(test)]
mod tests {

    #[cfg(test)]
    use pretty_assertions::assert_eq;
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

    #[test]
    fn test_minimum_transact_write() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let tx = ::raiden::WriteTx::new(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let cond = User::condition().attr_not_exists(User::id());
            let input = User::put_item_builder()
                .id("testId".to_owned())
                .name("bokuweb".to_owned())
                .build();
            let input2 = User::put_item_builder()
                .id("testId2".to_owned())
                .name("bokuweb".to_owned())
                .build();
            assert_eq!(
                tx.put(User::put(input).condition(cond))
                    .put(User::put(input2))
                    .run()
                    .await
                    .is_ok(),
                true,
            )
        }
        rt.block_on(example());
    }

    #[test]
    fn test_transact_write_put_and_update() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let tx = ::raiden::WriteTx::new(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let input = User::put_item_builder()
                .id("testId".to_owned())
                .name("bokuweb".to_owned())
                .build();
            let set_expression = User::update_expression()
                .set(User::name())
                .value("updated!!");

            let res = tx
                .put(User::put(input))
                .update(User::update("testId2").set(set_expression))
                .run()
                .await;
            assert_eq!(res.is_ok(), true);
        }
        rt.block_on(example());
    }

    #[test]
    fn test_transact_write_with_prefix_suffix() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let tx = ::raiden::WriteTx::new(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let input = User::put_item_builder()
                .id("testId".to_owned())
                .name("bokuweb".to_owned())
                .build();
            assert_eq!(
                tx.put(
                    User::put(input)
                        .table_prefix("test-")
                        .table_suffix("-staging"),
                )
                .run()
                .await
                .is_ok(),
                true,
            )
        }
        rt.block_on(example());
    }

    use std::sync::atomic::{AtomicUsize, Ordering};

    static RETRY_COUNT: AtomicUsize = AtomicUsize::new(0);
    struct MyRetryStrategy;

    impl RetryStrategy for MyRetryStrategy {
        fn should_retry(&self, _error: &RaidenError) -> bool {
            RETRY_COUNT.fetch_add(1, Ordering::Relaxed);
            true
        }

        fn policy(&self) -> Policy {
            Policy::Limit(3)
        }
    }

    #[test]
    fn test_retry() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let tx = ::raiden::WriteTx::new(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let input = User::put_item_builder()
                .id("testId".to_owned())
                .name("bokuweb".to_owned())
                .build();
            assert_eq!(
                tx.with_retries(Box::new(MyRetryStrategy))
                    .put(User::put(input).table_prefix("unknown"))
                    .run()
                    .await
                    .is_err(),
                true,
            )
        }
        rt.block_on(example());
        assert_eq!(RETRY_COUNT.load(Ordering::Relaxed), 4)
    }

    #[derive(Raiden, Debug, Clone, PartialEq)]
    pub struct TxDeleteTestData0 {
        #[raiden(partition_key)]
        id: String,
        name: String,
    }

    #[test]
    fn test_transact_delete_and_put() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let tx = ::raiden::WriteTx::new(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let input = TxDeleteTestData0::put_item_builder()
                .id("testId".to_owned())
                .name("bokuweb".to_owned())
                .build();
            assert_eq!(
                tx.put(TxDeleteTestData0::put(input))
                    .delete(TxDeleteTestData0::delete("id0"))
                    .run()
                    .await
                    .is_ok(),
                true,
            );

            let client = TxDeleteTestData0::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let res = client.get("id0").run().await;
            assert_eq!(
                res.unwrap_err(),
                RaidenError::ResourceNotFound("resource not found".to_owned())
            );
            let res = client.get("testId").run().await;
            assert_eq!(
                res.unwrap().item,
                TxDeleteTestData0 {
                    id: "testId".to_owned(),
                    name: "bokuweb".to_owned()
                }
            );
        }
        rt.block_on(example());
    }

    #[derive(Raiden, Debug, Clone, PartialEq)]
    pub struct TxConditionalCheckTestData0 {
        #[raiden(partition_key)]
        id: String,
        name: String,
    }

    #[derive(Raiden, Debug, Clone, PartialEq)]
    pub struct TxConditionalCheckTestData1 {
        #[raiden(partition_key)]
        id: String,
        name: String,
    }

    #[test]
    fn should_succeed_to_put_when_condition_check_ok() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let tx = ::raiden::WriteTx::new(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let input = TxConditionalCheckTestData0::put_item_builder()
                .id("testId0".to_owned())
                .name("bokuweb".to_owned())
                .build();
            let cond = TxConditionalCheckTestData1::condition()
                .attr_exists(TxConditionalCheckTestData1::id());
            assert_eq!(
                tx.put(TxConditionalCheckTestData0::put(input))
                    .condition_check(
                        TxConditionalCheckTestData1::condition_check("id1").condition(cond)
                    )
                    .run()
                    .await
                    .is_ok(),
                true,
            );

            let client = TxConditionalCheckTestData0::client(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let res = client.get("testId0").run().await;
            assert_eq!(
                res.unwrap().item,
                TxConditionalCheckTestData0 {
                    id: "testId0".to_owned(),
                    name: "bokuweb".to_owned()
                }
            );
        }
        rt.block_on(example());
    }

    #[test]
    fn should_fail_to_put_when_condition_check_ng() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        async fn example() {
            let tx = ::raiden::WriteTx::new(Region::Custom {
                endpoint: "http://localhost:8000".into(),
                name: "ap-northeast-1".into(),
            });
            let input = TxConditionalCheckTestData0::put_item_builder()
                .id("testId1".to_owned())
                .name("bokuweb".to_owned())
                .build();
            let cond = TxConditionalCheckTestData1::condition()
                .attr_not_exists(TxConditionalCheckTestData1::id());

            let res = tx
                .put(TxConditionalCheckTestData0::put(input))
                .condition_check(
                    TxConditionalCheckTestData1::condition_check("id1").condition(cond),
                )
                .run()
                .await;
            assert_eq!(res.is_err(), true,);
            assert_eq!(
                res.unwrap_err(),
                RaidenError::TransactionCanceled {
                    reasons: RaidenTransactionCancellationReasons(vec![
                        None,
                        Some(RaidenTransactionCancellationReason::ConditionalCheckFailed),
                    ]),
                }
            );
        }
        rt.block_on(example());
    }
}
