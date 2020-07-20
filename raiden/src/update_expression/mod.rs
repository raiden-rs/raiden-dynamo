pub mod add;
pub mod delete;
pub mod set;

pub use add::*;
pub use delete::*;
pub use set::*;

use super::{
    generate_value_id, AttributeNames, AttributeValue, AttributeValues, IntoAttrName,
    IntoAttribute, Placeholder,
};

pub enum SetOrRemove {
    Set(String, AttributeNames, AttributeValues),
    Remove(String, AttributeNames),
}

pub trait UpdateSetExpressionBuilder {
    fn build(self) -> SetOrRemove;
}

pub trait UpdateAddExpressionBuilder {
    fn build(self) -> (String, AttributeNames, AttributeValues);
}

pub trait UpdateDeleteExpressionBuilder {
    fn build(self) -> (String, AttributeNames, AttributeValues);
}
