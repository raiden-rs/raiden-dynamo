pub mod add;
pub mod set;
pub mod delete;

pub use add::*;
pub use set::*;
pub use delete::*;

use super::*;

pub trait UpdateExpressionBuilder {
    fn build(self) -> (String, super::AttributeNames, super::AttributeValues);
}
