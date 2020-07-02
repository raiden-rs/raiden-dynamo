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
                transact_items: vec![],
            });
        Ok(())
    }
}

// pub enum TransactWriteItem {
//     Put(crate::Put),
// }

/*
pub struct Put {
    table_name: String,
    table_prefix: String,
    table_suffix: String,
}

impl Put {
    pub fn new(table_name: impl Into<String>) -> Self {
        Self {
            table_name: table_name.into(),
            table_prefix: "".to_owned(),
            table_suffix: "".to_owned(),
        }
    }

    pub fn table_prefix(mut self, s: impl Into<String>) -> Self {
        self.table_prefix = s.into();
        self
    }

    pub fn table_suffix(mut self, s: impl Into<String>) -> Self {
        self.table_suffix = s.into();
        self
    }
}
*/

pub trait TransactWritePutBuilder {
    fn build(self) -> crate::Put;
}
