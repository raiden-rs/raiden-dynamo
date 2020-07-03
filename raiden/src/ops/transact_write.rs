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

    pub async fn run(self) -> Result<(), ()> {
        let res = self
            .client
            .transact_write_items(crate::TransactWriteItemsInput {
                client_request_token: None,
                return_consumed_capacity: None,
                return_item_collection_metrics: None,
                transact_items: self.items,
            })
            .await;
        dbg!(res);
        Ok(())
    }
}

pub trait TransactWritePutBuilder {
    fn build(self) -> crate::Put;
}
