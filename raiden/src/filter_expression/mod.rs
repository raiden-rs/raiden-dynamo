use super::*;

mod conversion_to_key_condition;

#[derive(Debug, Clone)]
pub enum FilterExpressionConjunction {
    And(
        KeyConditionString,
        super::AttributeNames,
        super::AttributeValues,
    ),
}

// note: The syntax for a filter expression is identical to that of a key condition expression.
// Filter expressions can use the same comparators, functions, and logical operators as a key condition expression, with the addition of the not-equals operator (<>).
// ref: https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Query.html
#[derive(Debug, Clone, PartialEq)]
pub enum FilterExpressionTypes {
    KeyConditionTypes(KeyConditionTypes),
    Not(super::Placeholder, super::AttributeValue),
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

#[derive(Debug, Clone)]
pub struct FilterExpressionFilledOrWaitConjunction<T> {
    attr: String,
    cond: FilterExpressionTypes,
    _token: std::marker::PhantomData<T>,
}

pub struct FilterExpressionFilled<T> {
    attr: String,
    cond: FilterExpressionTypes,
    conjunction: FilterExpressionConjunction,
    _token: std::marker::PhantomData<T>,
}

impl<T> FilterExpressionFilledOrWaitConjunction<T> {
    pub fn and(self, cond: impl FilterExpressionBuilder<T>) -> FilterExpressionFilled<T> {
        let (condition_string, attr_names, attr_values) = cond.build();
        FilterExpressionFilled {
            attr: self.attr,
            cond: self.cond,
            conjunction: FilterExpressionConjunction::And(
                condition_string,
                attr_names,
                attr_values,
            ),
            _token: self._token,
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
                    format!("#{} <> {}", attr_name, placeholder),
                    attr_names,
                    attr_values,
                )
            }
        }
    }
}

impl<T> FilterExpressionBuilder<T> for FilterExpressionFilled<T> {
    fn build(self) -> (String, super::AttributeNames, super::AttributeValues) {
        match self.cond {
            FilterExpressionTypes::KeyConditionTypes(_) => {
                Into::<KeyConditionFilled<_>>::into(self).build()
            }
            FilterExpressionTypes::Not(placeholder, value) => {
                let (right_str, right_names, right_values) = match self.conjunction {
                    FilterExpressionConjunction::And(s, m, v) => (format!("AND ({})", s), m, v),
                };

                let attr_name = self.attr;
                let mut left_names = super::AttributeNames::new();
                let mut left_values = super::AttributeValues::new();
                left_names.insert(format!("#{}", attr_name), attr_name.clone());
                left_values.insert(placeholder.clone(), value);
                let left_str = format!("#{}<> {}", attr_name, placeholder);
                (
                    format!("{} {}", left_str, right_str),
                    super::merge_map(left_names, right_names),
                    super::merge_map(left_values, right_values),
                )
            }
        }
    }
}

impl<T> FilterExpression<T> {
    pub fn eq(
        self,
        value: impl super::IntoAttribute,
    ) -> FilterExpressionFilledOrWaitConjunction<T> {
        Into::<KeyCondition<_>>::into(self).eq(value).into()
    }

    pub fn not(
        self,
        value: impl super::IntoAttribute,
    ) -> FilterExpressionFilledOrWaitConjunction<T> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let cond = FilterExpressionTypes::Not(placeholder, value.into_attr());
        FilterExpressionFilledOrWaitConjunction {
            attr: self.attr,
            cond,
            _token: std::marker::PhantomData,
        }
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
