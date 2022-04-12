pub type FilterExpressionString = String;

// note: The syntax for a filter expression is identical to that of a key condition expression.
// Filter expressions can use the same comparators, functions, and logical operators as a key condition expression, with the addition of the not-equals operator (<>).
// ref: https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Query.html
#[derive(Debug, Clone)]
pub enum FilterExpressionConjunction {
    And(
        FilterExpressionString,
        super::AttributeNames,
        super::AttributeValues,
    ),
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq)]
pub enum FilterExpressionTypes {
    Eq(super::Placeholder, super::AttributeValue),
    Not(super::Placeholder, super::AttributeValue),
    Le(super::Placeholder, super::AttributeValue),
    Ge(super::Placeholder, super::AttributeValue),
    Lt(super::Placeholder, super::AttributeValue),
    Gt(super::Placeholder, super::AttributeValue),
    Between(
        super::Placeholder,
        super::AttributeValue,
        super::Placeholder,
        super::AttributeValue,
    ),
    BeginsWith(super::Placeholder, super::AttributeValue),
}

pub trait FilterExpressionBuilder<T> {
    fn build(
        self,
    ) -> (
        FilterExpressionString,
        super::AttributeNames,
        super::AttributeValues,
    );
}

#[derive(Debug, Clone)]
pub struct FilterExpression<T> {
    pub attr: String,
    pub _token: std::marker::PhantomData<fn() -> T>,
}

#[derive(Debug, Clone)]
pub struct FilterExpressionFilledOrWaitConjunction<T> {
    attr: String,
    cond: FilterExpressionTypes,
    _token: std::marker::PhantomData<fn() -> T>,
}

#[derive(Debug, Clone)]
pub struct FilterExpressionFilled<T> {
    attr: String,
    cond: FilterExpressionTypes,
    conjunction: FilterExpressionConjunction,
    _token: std::marker::PhantomData<fn() -> T>,
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
        let attr_name = self.attr;
        let mut attr_names: super::AttributeNames = std::collections::HashMap::new();
        let mut attr_values: super::AttributeValues = std::collections::HashMap::new();

        attr_names.insert(format!("#{}", attr_name), attr_name.clone());
        match self.cond {
            FilterExpressionTypes::Eq(placeholder, value) => {
                attr_values.insert(placeholder.to_string(), value);
                (
                    format!("#{} = {}", attr_name, placeholder),
                    attr_names,
                    attr_values,
                )
            }
            FilterExpressionTypes::Not(placeholder, value) => {
                attr_values.insert(placeholder.to_string(), value);
                (
                    format!("#{} <> {}", attr_name, placeholder),
                    attr_names,
                    attr_values,
                )
            }
            FilterExpressionTypes::Gt(placeholder, value) => {
                attr_values.insert(placeholder.to_string(), value);
                (
                    format!("#{} > {}", attr_name, placeholder),
                    attr_names,
                    attr_values,
                )
            }
            FilterExpressionTypes::Ge(placeholder, value) => {
                attr_values.insert(placeholder.to_string(), value);
                (
                    format!("#{} >= {}", attr_name, placeholder),
                    attr_names,
                    attr_values,
                )
            }
            FilterExpressionTypes::Le(placeholder, value) => {
                attr_values.insert(placeholder.to_string(), value);
                (
                    format!("#{} <= {}", attr_name, placeholder),
                    attr_names,
                    attr_values,
                )
            }
            FilterExpressionTypes::Lt(placeholder, value) => {
                attr_values.insert(placeholder.to_string(), value);
                (
                    format!("#{} < {}", attr_name, placeholder),
                    attr_names,
                    attr_values,
                )
            }
            FilterExpressionTypes::Between(placeholder1, value1, placeholder2, value2) => {
                attr_values.insert(placeholder1.to_string(), value1);
                attr_values.insert(placeholder2.to_string(), value2);
                (
                    format!(
                        "#{} BETWEEN {} AND {}",
                        attr_name, placeholder1, placeholder2
                    ),
                    attr_names,
                    attr_values,
                )
            }
            FilterExpressionTypes::BeginsWith(placeholder, value) => {
                attr_values.insert(placeholder.to_string(), value);
                (
                    format!("begins_with(#{}, {})", attr_name, placeholder),
                    attr_names,
                    attr_values,
                )
            }
        }
    }
}

impl<T> FilterExpressionBuilder<T> for FilterExpressionFilled<T> {
    fn build(self) -> (String, super::AttributeNames, super::AttributeValues) {
        let (right_str, right_names, right_values) = match self.conjunction {
            FilterExpressionConjunction::And(s, m, v) => (format!("AND ({})", s), m, v),
        };

        let attr_name = self.attr;
        let mut left_names: super::AttributeNames = std::collections::HashMap::new();
        let mut left_values: super::AttributeValues = std::collections::HashMap::new();
        left_names.insert(format!("#{}", attr_name), attr_name.clone());

        let left_str = match self.cond {
            FilterExpressionTypes::Eq(placeholder, value) => {
                left_values.insert(placeholder.clone(), value);
                format!("#{} = {}", attr_name, placeholder)
            }
            FilterExpressionTypes::Not(placeholder, value) => {
                left_values.insert(placeholder.clone(), value);
                format!("#{} <> {}", attr_name, placeholder)
            }
            FilterExpressionTypes::Gt(placeholder, value) => {
                left_values.insert(placeholder.clone(), value);
                format!("#{} > {}", attr_name, placeholder)
            }
            FilterExpressionTypes::Ge(placeholder, value) => {
                left_values.insert(placeholder.clone(), value);
                format!("#{} >= {}", attr_name, placeholder)
            }
            FilterExpressionTypes::Le(placeholder, value) => {
                left_values.insert(placeholder.clone(), value);
                format!("#{} <= {}", attr_name, placeholder)
            }
            FilterExpressionTypes::Lt(placeholder, value) => {
                left_values.insert(placeholder.clone(), value);
                format!("#{} < {}", attr_name, placeholder)
            }
            FilterExpressionTypes::Between(placeholder1, value1, placeholder2, value2) => {
                left_values.insert(placeholder1.clone(), value1);
                left_values.insert(placeholder2.clone(), value2);
                format!(
                    "#{} BETWEEN {} AND {}",
                    attr_name, placeholder1, placeholder2
                )
            }
            FilterExpressionTypes::BeginsWith(placeholder, value) => {
                left_values.insert(placeholder.clone(), value);
                format!("begins_with(#{}, {})", attr_name, placeholder)
            }
        };
        (
            format!("{} {}", left_str, right_str),
            super::merge_map(left_names, right_names),
            super::merge_map(left_values, right_values),
        )
    }
}

impl<T> FilterExpression<T> {
    pub fn eq(
        self,
        value: impl super::IntoAttribute,
    ) -> FilterExpressionFilledOrWaitConjunction<T> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let cond = FilterExpressionTypes::Eq(placeholder, value.into_attr());
        FilterExpressionFilledOrWaitConjunction {
            attr: self.attr,
            cond,
            _token: std::marker::PhantomData,
        }
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
        let placeholder = format!(":value{}", super::generate_value_id());
        let cond = FilterExpressionTypes::Gt(placeholder, value.into_attr());
        FilterExpressionFilledOrWaitConjunction {
            attr: self.attr,
            cond,
            _token: std::marker::PhantomData,
        }
    }
    pub fn ge(
        self,
        value: impl super::IntoAttribute,
    ) -> FilterExpressionFilledOrWaitConjunction<T> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let cond = FilterExpressionTypes::Ge(placeholder, value.into_attr());
        FilterExpressionFilledOrWaitConjunction {
            attr: self.attr,
            cond,
            _token: std::marker::PhantomData,
        }
    }

    pub fn le(
        self,
        value: impl super::IntoAttribute,
    ) -> FilterExpressionFilledOrWaitConjunction<T> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let cond = FilterExpressionTypes::Le(placeholder, value.into_attr());
        FilterExpressionFilledOrWaitConjunction {
            attr: self.attr,
            cond,
            _token: std::marker::PhantomData,
        }
    }

    pub fn lt(
        self,
        value: impl super::IntoAttribute,
    ) -> FilterExpressionFilledOrWaitConjunction<T> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let cond = FilterExpressionTypes::Lt(placeholder, value.into_attr());
        FilterExpressionFilledOrWaitConjunction {
            attr: self.attr,
            cond,
            _token: std::marker::PhantomData,
        }
    }

    pub fn between(
        self,
        value1: impl super::IntoAttribute,
        value2: impl super::IntoAttribute,
    ) -> FilterExpressionFilledOrWaitConjunction<T> {
        let placeholder1 = format!(":value{}", super::generate_value_id());
        let placeholder2 = format!(":value{}", super::generate_value_id());
        let cond = FilterExpressionTypes::Between(
            placeholder1,
            value1.into_attr(),
            placeholder2,
            value2.into_attr(),
        );
        FilterExpressionFilledOrWaitConjunction {
            attr: self.attr,
            cond,
            _token: std::marker::PhantomData,
        }
    }

    // We can use `begins_with` only with a range key after specifying an EQ condition for the primary key.
    pub fn begins_with(
        self,
        value: impl super::IntoAttribute,
    ) -> FilterExpressionFilledOrWaitConjunction<T> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let cond = FilterExpressionTypes::BeginsWith(placeholder, value.into_attr());
        FilterExpressionFilledOrWaitConjunction {
            attr: self.attr,
            cond,
            _token: std::marker::PhantomData,
        }
    }
}
