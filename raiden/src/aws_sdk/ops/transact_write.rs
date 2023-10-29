// DynamoDb, DynamoDbClient, TransactWriteItem, TransactWriteItemsInput
use crate::{
    Client, Config, RaidenError, Region, RetryCondition, RetryStrategy,
    TransactWriteConditionCheckBuilder, TransactWriteDeleteBuilder, TransactWriteItem,
    TransactWriteItemsFluentBuilder, TransactWritePutBuilder, TransactWriteUpdateBuilder,
};

pub struct WriteTx {
    items: Vec<TransactWriteItem>,
    client: Client,
    retry_condition: RetryCondition,
}

impl WriteTx {
    pub fn new(region: Region) -> Self {
        let config = Config::builder().region(region).build();

        Self {
            items: vec![],
            client: Client::from_conf(config),
            retry_condition: RetryCondition::new(),
        }
    }

    pub fn new_with_client(client: Client) -> Self {
        Self {
            items: vec![],
            client,
            retry_condition: RetryCondition::new(),
        }
    }

    pub fn with_retries(mut self, s: Box<dyn RetryStrategy + Send + Sync>) -> Self {
        self.retry_condition.strategy = s;
        self
    }

    pub fn put(mut self, builder: impl TransactWritePutBuilder) -> Self {
        let builder = TransactWriteItem::builder().put(builder.build());

        self.items.push(builder.build());
        self
    }

    pub fn update(mut self, builder: impl TransactWriteUpdateBuilder) -> Self {
        let builder = TransactWriteItem::builder().update(builder.build());

        self.items.push(builder.build());
        self
    }

    pub fn delete(mut self, builder: impl TransactWriteDeleteBuilder) -> Self {
        let builder = TransactWriteItem::builder().delete(builder.build());

        self.items.push(builder.build());
        self
    }

    pub fn condition_check(mut self, builder: impl TransactWriteConditionCheckBuilder) -> Self {
        let builder = TransactWriteItem::builder().condition_check(builder.build());

        self.items.push(builder.build());
        self
    }

    pub async fn run(self) -> Result<(), RaidenError> {
        let policy: crate::RetryPolicy = self.retry_condition.strategy.policy().into();
        let req = self
            .client
            .transact_write_items()
            .set_transact_items(Some(self.items));

        policy
            .retry_if(
                move || {
                    let req = req.clone();
                    async { WriteTx::inner_run(req).await }
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
    async fn inner_run(req: TransactWriteItemsFluentBuilder) -> Result<(), RaidenError> {
        let _res = req.send().await?;

        // TODO: ADD Resp
        Ok(())
    }
}
