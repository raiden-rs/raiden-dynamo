#[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
use crate::{ConditionCheck, Delete, Put, Update};

#[cfg(feature = "aws-sdk")]
use crate::aws_sdk::types::{ConditionCheck, Delete, Put, Update};

pub trait TransactWritePutBuilder {
    fn build(self) -> Put;
}

pub trait TransactWriteUpdateBuilder {
    fn build(self) -> Update;
}

pub trait TransactWriteDeleteBuilder {
    fn build(self) -> Delete;
}

pub trait TransactWriteConditionCheckBuilder {
    fn build(self) -> ConditionCheck;
}
