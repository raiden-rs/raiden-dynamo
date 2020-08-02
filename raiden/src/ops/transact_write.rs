use crate::{DynamoDb, TransactWriteItem};

pub struct WriteTx {
    items: Vec<crate::TransactWriteItem>,
    client: crate::DynamoDbClient,
}
impl WriteTx {
    pub fn new(region: crate::Region) -> Self {
        let client = crate::DynamoDbClient::new(region);
        Self {
            items: vec![],
            client,
        }
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

    pub async fn run(self) -> Result<(), crate::RaidenError> {
        let _res = self
            .client
            .transact_write_items(crate::TransactWriteItemsInput {
                client_request_token: None,
                return_consumed_capacity: None,
                return_item_collection_metrics: None,
                transact_items: self.items,
            })
            .await?;
        // TODO: ADD Response later
        dbg!(&_res);
        Ok(())
    }
}

pub trait TransactWritePutBuilder {
    fn build(self) -> crate::Put;
}

pub trait TransactWriteUpdateBuilder {
    fn build(self) -> crate::Update;
}
