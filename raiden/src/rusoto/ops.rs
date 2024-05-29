pub use transact_write::*;

mod transact_write {
    use crate::{
        DynamoDb, TransactWriteConditionCheckBuilder, TransactWriteDeleteBuilder,
        TransactWriteItem, TransactWritePutBuilder, TransactWriteUpdateBuilder,
    };

    pub struct WriteTx {
        items: Vec<crate::TransactWriteItem>,
        client: crate::DynamoDbClient,
        retry_condition: crate::RetryCondition,
    }

    impl WriteTx {
        pub fn new(region: crate::Region) -> Self {
            Self {
                items: vec![],
                client: crate::DynamoDbClient::new(region),
                retry_condition: crate::RetryCondition::new(),
            }
        }

        pub fn new_with_client(client: crate::Client, region: crate::Region) -> Self {
            Self {
                items: vec![],
                client: crate::DynamoDbClient::new_with_client(client, region),
                retry_condition: crate::RetryCondition::new(),
            }
        }

        pub fn with_retries(
            mut self,
            s: Box<dyn crate::retry::RetryStrategy + Send + Sync>,
        ) -> Self {
            self.retry_condition.strategy = s;
            self
        }

        pub fn put(mut self, builder: impl TransactWritePutBuilder) -> Self {
            self.items.push(TransactWriteItem {
                condition_check: None,
                delete: None,
                update: None,
                put: Some(builder.build()),
            });
            self
        }

        pub fn update(mut self, builder: impl TransactWriteUpdateBuilder) -> Self {
            self.items.push(TransactWriteItem {
                condition_check: None,
                delete: None,
                update: Some(builder.build()),
                put: None,
            });
            self
        }

        pub fn delete(mut self, builder: impl TransactWriteDeleteBuilder) -> Self {
            self.items.push(TransactWriteItem {
                condition_check: None,
                delete: Some(builder.build()),
                update: None,
                put: None,
            });
            self
        }

        pub fn condition_check(mut self, builder: impl TransactWriteConditionCheckBuilder) -> Self {
            self.items.push(TransactWriteItem {
                condition_check: Some(builder.build()),
                delete: None,
                update: None,
                put: None,
            });
            self
        }

        pub async fn run(self) -> Result<(), crate::RaidenError> {
            let policy: crate::RetryPolicy = self.retry_condition.strategy.policy().into();
            let client = self.client;
            let input = crate::TransactWriteItemsInput {
                client_request_token: None,
                return_consumed_capacity: None,
                return_item_collection_metrics: None,
                transact_items: self.items,
            };

            policy
                .retry_if(
                    move || {
                        let client = client.clone();
                        let input = input.clone();
                        async { WriteTx::inner_run(client, input).await }
                    },
                    &self.retry_condition,
                )
                .await
        }

        #[cfg_attr(feature = "tracing", tracing::instrument(
            level = tracing::Level::DEBUG,
            name = "dynamodb::action",
            skip_all,
            fields(api = "transact_write_items")
        ))]
        async fn inner_run(
            client: crate::DynamoDbClient,
            input: crate::TransactWriteItemsInput,
        ) -> Result<(), crate::RaidenError> {
            let _res = client.transact_write_items(input).await?;
            // TODO: ADD Resp
            Ok(())
        }
    }
}
