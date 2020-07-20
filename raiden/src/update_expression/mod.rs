pub mod add;
pub mod delete;
pub mod set;

pub use add::*;
pub use delete::*;
pub use set::*;

use super::{
    generate_value_id, AttributeNames, AttributeValue, AttributeValues, IntoAttrName, IntoAttribute, Placeholder
};

pub trait UpdateExpressionBuilder {
    fn build(self) -> (String, AttributeNames, AttributeValues);
}
