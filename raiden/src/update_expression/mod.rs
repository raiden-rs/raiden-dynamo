pub mod add;
pub mod set;

pub use add::*;
pub use set::*;

use super::*;

pub trait SetExpressionBuilder {
    fn build(self) -> (String, super::AttributeNames, super::AttributeValues);
}
