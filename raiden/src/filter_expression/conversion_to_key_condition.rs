use crate::{
    KeyCondition, KeyConditionConjunction, KeyConditionFilled, KeyConditionFilledOrWaitConjunction,
    KeyConditionTypes,
};

use super::{
    FilterExpression, FilterExpressionConjunction, FilterExpressionFilled,
    FilterExpressionFilledOrWaitConjunction, FilterExpressionTypes,
};

impl<T> Into<KeyCondition<T>> for FilterExpression<T> {
    fn into(self) -> KeyCondition<T> {
        KeyCondition::new(self.attr, std::marker::PhantomData)
    }
}

impl Into<KeyConditionConjunction> for FilterExpressionConjunction {
    fn into(self) -> KeyConditionConjunction {
        match self {
            FilterExpressionConjunction::And(s, attr_name, attr_value) => {
                KeyConditionConjunction::And(s, attr_name, attr_value)
            }
        }
    }
}

impl<T> Into<KeyConditionFilled<T>> for FilterExpressionFilled<T> {
    fn into(self) -> KeyConditionFilled<T> {
        let cond = self.cond.into();
        let conjunction = self.conjunction.into();
        KeyConditionFilled::new(self.attr, cond, conjunction, std::marker::PhantomData)
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

impl From<KeyConditionTypes> for FilterExpressionTypes {
    fn from(v: KeyConditionTypes) -> Self {
        Self::KeyConditionTypes(v)
    }
}

// FIXME: consider using TryInto
// note: avoid this conversion in case that you don't confirm the FilterExpressionTypes is KeyConditionTypes
impl Into<KeyConditionTypes> for FilterExpressionTypes {
    fn into(self) -> KeyConditionTypes {
        match self {
            Self::KeyConditionTypes(k) => k,
            _ => unimplemented!(),
        }
    }
}
