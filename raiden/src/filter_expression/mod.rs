use super::*;

// note: The syntax for a filter expression is identical to that of a key condition expression.
// Filter expressions can use the same comparators, functions, and logical operators as a key condition expression, with the addition of the not-equals operator (<>).
// ref: https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Query.html
#[derive(Debug, Clone, PartialEq)]
pub enum FilterExpressionTypes {
    KeyConditionTypes(KeyConditionTypes),
    Not(super::Placeholder, super::AttributeValue),
}

impl From<KeyConditionTypes> for FilterExpressionTypes {
    fn from(v: KeyConditionTypes) -> Self {
        Self::KeyConditionTypes(v)
    }
}

// FIXME: consider using TryInto
impl Into<KeyConditionTypes> for FilterExpressionTypes {
    fn into(self) -> KeyConditionTypes {
        match self {
            Self::KeyConditionTypes(k) => k,
            _ => unimplemented!(),
        }
    }
}

pub trait FilterExpressionBuilder<T> {
    fn build(
        self,
    ) -> (
        KeyConditionString,
        super::AttributeNames,
        super::AttributeValues,
    );
}

#[derive(Debug, Clone)]
pub struct FilterExpression<T> {
    pub attr: String,
    pub _token: std::marker::PhantomData<T>,
}

impl<T> Into<KeyCondition<T>> for FilterExpression<T> {
    fn into(self) -> KeyCondition<T> {
        KeyCondition::new(self.attr, std::marker::PhantomData)
    }
}

#[derive(Debug, Clone)]
pub struct FilterExpresssionFilled<T> {
    attr: String,
    cond: FilterExpressionTypes,
    conjunction: FilterExpressionConjunction,
    _token: std::marker::PhantomData<T>,
}

#[derive(Debug, Clone)]
pub enum FilterExpressionConjunction {
    And(
        KeyConditionString,
        super::AttributeNames,
        super::AttributeValues,
    ),
}

#[derive(Debug, Clone)]
pub struct FilterExpressionFilledOrWaitConjunction<T> {
    attr: String,
    cond: FilterExpressionTypes,
    _token: std::marker::PhantomData<T>,
}

impl<T> From<KeyConditionFilledOrWaitConjunction<T>>
    for FilterExpressionFilledOrWaitConjunction<T>
{
    fn from(v: KeyConditionFilledOrWaitConjunction<T>) -> Self {
        Self {
            attr: v.attr,
            cond: FilterExpressionTypes::from(v.cond),
            _token: std::marker::PhantomData,
        }
    }
}

impl<T> FilterExpressionBuilder<T> for FilterExpressionFilledOrWaitConjunction<T> {
    fn build(self) -> (String, super::AttributeNames, super::AttributeValues) {
        match self.cond {
            FilterExpressionTypes::KeyConditionTypes(_) => {
                Into::<KeyConditionFilledOrWaitConjunction<_>>::into(self).build()
            }
            FilterExpressionTypes::Not(placeholder, value) => {
                let attr_name = self.attr;
                let mut attr_names = super::AttributeNames::new();
                let mut attr_values = super::AttributeValues::new();

                attr_names.insert(format!("#{}", attr_name), attr_name.clone());
                attr_values.insert(placeholder.to_string(), value);
                (
                    format!("#{} = {}", attr_name, placeholder),
                    attr_names,
                    attr_values,
                )
            }
        }
    }
}

impl<T> Into<KeyConditionFilledOrWaitConjunction<T>>
    for FilterExpressionFilledOrWaitConjunction<T>
{
    fn into(self) -> KeyConditionFilledOrWaitConjunction<T> {
        KeyConditionFilledOrWaitConjunction::new(
            self.attr,
            self.cond.into(),
            std::marker::PhantomData,
        )
    }
}

macro_rules! filter_expression_impl_inner {
    ($op:ident; $($value:ident),+) => {
        impl<T> FilterExpression<T> {
            pub fn $op(
                self,
                $($value: impl super::IntoAttribute,)+
            ) -> FilterExpressionFilledOrWaitConjunction<T> {
                Into::<KeyCondition<_>>::into(self).$op($($value,)+).into()
            }
        }
    };
    (@unary $op:ident) => {
        filter_expression_impl_inner!($op; value1);
    };
    (@binary $op:ident) => {
        filter_expression_impl_inner!($op; value1, value2);
    };
}

macro_rules! filter_expression_impl {
    ($($args:tt),+) => {
        $(filter_expression_impl_inner!(@unary $args);)+
    };
}

filter_expression_impl!(eq, gt, ge, le, lt, begins_with);
filter_expression_impl_inner!(@binary between);
