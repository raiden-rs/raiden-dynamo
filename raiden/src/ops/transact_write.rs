pub trait TransactWritePutBuilder {
    fn build(self) -> crate::Put;
}

pub trait TransactWriteUpdateBuilder {
    fn build(self) -> crate::Update;
}

pub trait TransactWriteDeleteBuilder {
    fn build(self) -> crate::Delete;
}

pub trait TransactWriteConditionCheckBuilder {
    fn build(self) -> crate::ConditionCheck;
}
