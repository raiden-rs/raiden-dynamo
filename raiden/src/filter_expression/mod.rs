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

impl<T> FilterExpression<T> {
    // TODO: commonalize these methods by macro
    pub fn eq(
        self,
        value: impl super::IntoAttribute,
    ) -> FilterExpressionFilledOrWaitConjunction<T> {
        Into::<KeyCondition<_>>::into(self).eq(value).into()
    }

    pub fn gt(
        self,
        value: impl super::IntoAttribute,
    ) -> FilterExpressionFilledOrWaitConjunction<T> {
        Into::<KeyCondition<_>>::into(self).gt(value).into()
    }

    pub fn ge(
        self,
        value: impl super::IntoAttribute,
    ) -> FilterExpressionFilledOrWaitConjunction<T> {
        Into::<KeyCondition<_>>::into(self).ge(value).into()
    }

    pub fn le(
        self,
        value: impl super::IntoAttribute,
    ) -> FilterExpressionFilledOrWaitConjunction<T> {
        Into::<KeyCondition<_>>::into(self).le(value).into()
    }

    pub fn lt(
        self,
        value: impl super::IntoAttribute,
    ) -> FilterExpressionFilledOrWaitConjunction<T> {
        Into::<KeyCondition<_>>::into(self).lt(value).into()
    }

    pub fn between(
        self,
        value1: impl super::IntoAttribute,
        value2: impl super::IntoAttribute,
    ) -> FilterExpressionFilledOrWaitConjunction<T> {
        Into::<KeyCondition<_>>::into(self)
            .between(value1, value2)
            .into()
    }

    // We can use `begins_with` only with a range key after specifying an EQ condition for the primary key.
    pub fn begins_with(
        self,
        value: impl super::IntoAttribute,
    ) -> FilterExpressionFilledOrWaitConjunction<T> {
        Into::<KeyCondition<_>>::into(self)
            .begins_with(value)
            .into()
    }
}
