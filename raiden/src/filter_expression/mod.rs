use crate::IntoAttribute;

pub type FilterExpressionString = String;

// note: The syntax for a filter expression is identical to that of a key condition expression.
// Filter expressions can use the same comparators, functions, and logical operators as a key condition expression, with the addition of the not-equals operator (<>).
// ref: https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Query.html
#[derive(Debug, Clone)]
pub enum FilterExpressionOperator {
    And(
        FilterExpressionString,
        super::AttributeNames,
        super::AttributeValues,
    ),
    Or(
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
    In(Vec<(super::Placeholder, super::AttributeValue)>),
    BeginsWith(super::Placeholder, super::AttributeValue),
    AttributeExists(),
    AttributeNotExists(),
    AttributeType(super::Placeholder, super::AttributeType),
    Contains(super::Placeholder, super::AttributeValue),
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
    pub is_size: bool,
    pub _token: std::marker::PhantomData<fn() -> T>,
}

#[derive(Debug, Clone)]
pub struct FilterExpressionFilledOrWaitOperator<T> {
    attr: String,
    is_size: bool,
    cond: FilterExpressionTypes,
    _token: std::marker::PhantomData<fn() -> T>,
}

#[derive(Debug, Clone)]
pub struct FilterExpressionFilled<T> {
    attr: String,
    is_size: bool,
    cond: FilterExpressionTypes,
    operator: FilterExpressionOperator,
    _token: std::marker::PhantomData<fn() -> T>,
}

impl<T> FilterExpressionFilledOrWaitOperator<T> {
    pub fn and(self, cond: impl FilterExpressionBuilder<T>) -> FilterExpressionFilled<T> {
        let (condition_string, attr_names, attr_values) = cond.build();
        FilterExpressionFilled {
            attr: self.attr,
            is_size: self.is_size,
            cond: self.cond,
            operator: FilterExpressionOperator::And(condition_string, attr_names, attr_values),
            _token: self._token,
        }
    }
    pub fn or(self, cond: impl FilterExpressionBuilder<T>) -> FilterExpressionFilled<T> {
        let (condition_string, attr_names, attr_values) = cond.build();
        FilterExpressionFilled {
            attr: self.attr,
            is_size: self.is_size,
            cond: self.cond,
            operator: FilterExpressionOperator::Or(condition_string, attr_names, attr_values),
            _token: self._token,
        }
    }
}

impl<T> FilterExpressionBuilder<T> for FilterExpressionFilledOrWaitOperator<T> {
    fn build(self) -> (String, super::AttributeNames, super::AttributeValues) {
        let attr_name = self.attr;
        let mut attr_names: super::AttributeNames = std::collections::HashMap::new();
        let mut attr_values: super::AttributeValues = std::collections::HashMap::new();

        attr_names.insert(format!("#{}", attr_name), attr_name.clone());
        let left_cond = if self.is_size {
            format!("size(#{})", attr_name)
        } else {
            format!("#{}", attr_name)
        };
        match self.cond {
            FilterExpressionTypes::Eq(placeholder, value) => {
                attr_values.insert(placeholder.to_string(), value);
                (
                    format!("{} = {}", left_cond, placeholder),
                    attr_names,
                    attr_values,
                )
            }
            FilterExpressionTypes::Not(placeholder, value) => {
                attr_values.insert(placeholder.to_string(), value);
                (
                    format!("{} <> {}", left_cond, placeholder),
                    attr_names,
                    attr_values,
                )
            }
            FilterExpressionTypes::Gt(placeholder, value) => {
                attr_values.insert(placeholder.to_string(), value);
                (
                    format!("{} > {}", left_cond, placeholder),
                    attr_names,
                    attr_values,
                )
            }
            FilterExpressionTypes::Ge(placeholder, value) => {
                attr_values.insert(placeholder.to_string(), value);
                (
                    format!("{} >= {}", left_cond, placeholder),
                    attr_names,
                    attr_values,
                )
            }
            FilterExpressionTypes::Le(placeholder, value) => {
                attr_values.insert(placeholder.to_string(), value);
                (
                    format!("{} <= {}", left_cond, placeholder),
                    attr_names,
                    attr_values,
                )
            }
            FilterExpressionTypes::Lt(placeholder, value) => {
                attr_values.insert(placeholder.to_string(), value);
                (
                    format!("{} < {}", left_cond, placeholder),
                    attr_names,
                    attr_values,
                )
            }
            FilterExpressionTypes::Between(placeholder1, value1, placeholder2, value2) => {
                attr_values.insert(placeholder1.to_string(), value1);
                attr_values.insert(placeholder2.to_string(), value2);
                (
                    format!(
                        "{} BETWEEN {} AND {}",
                        left_cond, placeholder1, placeholder2
                    ),
                    attr_names,
                    attr_values,
                )
            }
            FilterExpressionTypes::In(attributes) => {
                let placeholders = attributes
                    .iter()
                    .map(|(placeholder, _)| placeholder.clone())
                    .collect::<Vec<_>>()
                    .join(",");
                for (placeholder, value) in attributes {
                    attr_values.insert(placeholder, value);
                }
                (
                    format!("{} IN ({})", left_cond, placeholders),
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
            FilterExpressionTypes::AttributeExists() => (
                format!("attribute_exists(#{})", attr_name),
                attr_names,
                attr_values,
            ),
            FilterExpressionTypes::AttributeNotExists() => (
                format!("attribute_not_exists(#{})", attr_name),
                attr_names,
                attr_values,
            ),
            FilterExpressionTypes::AttributeType(placeholder, attribute_type) => {
                attr_values.insert(placeholder.to_string(), attribute_type.into_attr());
                (
                    format!("attribute_type(#{}, {})", attr_name, placeholder),
                    attr_names,
                    attr_values,
                )
            }
            FilterExpressionTypes::Contains(placeholder, value) => {
                attr_values.insert(placeholder.to_string(), value);
                (
                    format!("contains(#{}, {})", attr_name, placeholder),
                    attr_names,
                    attr_values,
                )
            }
        }
    }
}

impl<T> FilterExpressionBuilder<T> for FilterExpressionFilled<T> {
    fn build(self) -> (String, super::AttributeNames, super::AttributeValues) {
        let (right_str, right_names, right_values) = match self.operator {
            FilterExpressionOperator::And(s, m, v) => (format!("AND ({})", s), m, v),
            FilterExpressionOperator::Or(s, m, v) => (format!("OR ({})", s), m, v),
        };

        let attr_name = self.attr;
        let mut left_names: super::AttributeNames = std::collections::HashMap::new();
        let mut left_values: super::AttributeValues = std::collections::HashMap::new();
        left_names.insert(format!("#{}", attr_name), attr_name.clone());
        let left_cond = if self.is_size {
            format!("size(#{})", attr_name)
        } else {
            format!("#{}", attr_name)
        };

        let left_str = match self.cond {
            FilterExpressionTypes::Eq(placeholder, value) => {
                left_values.insert(placeholder.clone(), value);
                format!("{} = {}", left_cond, placeholder)
            }
            FilterExpressionTypes::Not(placeholder, value) => {
                left_values.insert(placeholder.clone(), value);
                format!("{} <> {}", left_cond, placeholder)
            }
            FilterExpressionTypes::Gt(placeholder, value) => {
                left_values.insert(placeholder.clone(), value);
                format!("{} > {}", left_cond, placeholder)
            }
            FilterExpressionTypes::Ge(placeholder, value) => {
                left_values.insert(placeholder.clone(), value);
                format!("{} >= {}", left_cond, placeholder)
            }
            FilterExpressionTypes::Le(placeholder, value) => {
                left_values.insert(placeholder.clone(), value);
                format!("{} <= {}", left_cond, placeholder)
            }
            FilterExpressionTypes::Lt(placeholder, value) => {
                left_values.insert(placeholder.clone(), value);
                format!("{} < {}", left_cond, placeholder)
            }
            FilterExpressionTypes::Between(placeholder1, value1, placeholder2, value2) => {
                left_values.insert(placeholder1.clone(), value1);
                left_values.insert(placeholder2.clone(), value2);
                format!(
                    "{} BETWEEN {} AND {}",
                    left_cond, placeholder1, placeholder2
                )
            }
            FilterExpressionTypes::In(attributes) => {
                let placeholders = attributes
                    .iter()
                    .map(|(placeholder, _)| placeholder.clone())
                    .collect::<Vec<_>>()
                    .join(",");
                for (placeholder, value) in attributes {
                    left_values.insert(placeholder, value);
                }
                format!("{} IN ({})", attr_name, placeholders)
            }
            FilterExpressionTypes::BeginsWith(placeholder, value) => {
                left_values.insert(placeholder.clone(), value);
                format!("begins_with(#{}, {})", attr_name, placeholder)
            }
            FilterExpressionTypes::AttributeExists() => {
                format!("attribute_exists(#{})", attr_name)
            }
            FilterExpressionTypes::AttributeNotExists() => {
                format!("attribute_not_exists(#{})", attr_name)
            }
            FilterExpressionTypes::AttributeType(placeholder, attribute_type) => {
                left_values.insert(placeholder.clone(), attribute_type.into_attr());
                format!("attribute_type(#{}, {})", attr_name, placeholder)
            }
            FilterExpressionTypes::Contains(placeholder, value) => {
                left_values.insert(placeholder.clone(), value);
                format!("contains(#{}, {})", attr_name, placeholder)
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
    pub fn size(mut self) -> Self {
        self.is_size = true;
        self
    }

    pub fn eq(self, value: impl super::IntoAttribute) -> FilterExpressionFilledOrWaitOperator<T> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let cond = FilterExpressionTypes::Eq(placeholder, value.into_attr());
        FilterExpressionFilledOrWaitOperator {
            attr: self.attr,
            is_size: self.is_size,
            cond,
            _token: std::marker::PhantomData,
        }
    }

    pub fn not(self, value: impl super::IntoAttribute) -> FilterExpressionFilledOrWaitOperator<T> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let cond = FilterExpressionTypes::Not(placeholder, value.into_attr());
        FilterExpressionFilledOrWaitOperator {
            attr: self.attr,
            is_size: self.is_size,
            cond,
            _token: std::marker::PhantomData,
        }
    }

    pub fn gt(self, value: impl super::IntoAttribute) -> FilterExpressionFilledOrWaitOperator<T> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let cond = FilterExpressionTypes::Gt(placeholder, value.into_attr());
        FilterExpressionFilledOrWaitOperator {
            attr: self.attr,
            is_size: self.is_size,
            cond,
            _token: std::marker::PhantomData,
        }
    }
    pub fn ge(self, value: impl super::IntoAttribute) -> FilterExpressionFilledOrWaitOperator<T> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let cond = FilterExpressionTypes::Ge(placeholder, value.into_attr());
        FilterExpressionFilledOrWaitOperator {
            attr: self.attr,
            is_size: self.is_size,
            cond,
            _token: std::marker::PhantomData,
        }
    }

    pub fn le(self, value: impl super::IntoAttribute) -> FilterExpressionFilledOrWaitOperator<T> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let cond = FilterExpressionTypes::Le(placeholder, value.into_attr());
        FilterExpressionFilledOrWaitOperator {
            attr: self.attr,
            is_size: self.is_size,
            cond,
            _token: std::marker::PhantomData,
        }
    }

    pub fn lt(self, value: impl super::IntoAttribute) -> FilterExpressionFilledOrWaitOperator<T> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let cond = FilterExpressionTypes::Lt(placeholder, value.into_attr());
        FilterExpressionFilledOrWaitOperator {
            attr: self.attr,
            is_size: self.is_size,
            cond,
            _token: std::marker::PhantomData,
        }
    }

    pub fn between(
        self,
        value1: impl super::IntoAttribute,
        value2: impl super::IntoAttribute,
    ) -> FilterExpressionFilledOrWaitOperator<T> {
        let placeholder1 = format!(":value{}", super::generate_value_id());
        let placeholder2 = format!(":value{}", super::generate_value_id());
        let cond = FilterExpressionTypes::Between(
            placeholder1,
            value1.into_attr(),
            placeholder2,
            value2.into_attr(),
        );
        FilterExpressionFilledOrWaitOperator {
            attr: self.attr,
            is_size: self.is_size,
            cond,
            _token: std::marker::PhantomData,
        }
    }

    pub fn r#in(
        self,
        values: Vec<impl super::IntoAttribute>,
    ) -> FilterExpressionFilledOrWaitOperator<T> {
        let attributes = values.into_iter().map(|value| {
            let placeholder = format!(":value{}", super::generate_value_id());
            (placeholder, value.into_attr())
        });
        let cond = FilterExpressionTypes::In(attributes.collect());
        FilterExpressionFilledOrWaitOperator {
            attr: self.attr,
            is_size: self.is_size,
            cond,
            _token: std::marker::PhantomData,
        }
    }

    // We can use `begins_with` only with a range key after specifying an EQ condition for the primary key.
    pub fn begins_with(
        self,
        value: impl super::IntoAttribute,
    ) -> FilterExpressionFilledOrWaitOperator<T> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let cond = FilterExpressionTypes::BeginsWith(placeholder, value.into_attr());
        FilterExpressionFilledOrWaitOperator {
            attr: self.attr,
            is_size: self.is_size,
            cond,
            _token: std::marker::PhantomData,
        }
    }

    pub fn attribute_exists(self) -> FilterExpressionFilledOrWaitOperator<T> {
        let cond = FilterExpressionTypes::AttributeExists();
        FilterExpressionFilledOrWaitOperator {
            attr: self.attr,
            is_size: self.is_size,
            cond,
            _token: std::marker::PhantomData,
        }
    }

    pub fn attribute_not_exists(self) -> FilterExpressionFilledOrWaitOperator<T> {
        let cond = FilterExpressionTypes::AttributeNotExists();
        FilterExpressionFilledOrWaitOperator {
            attr: self.attr,
            is_size: self.is_size,
            cond,
            _token: std::marker::PhantomData,
        }
    }

    pub fn attribute_type(
        self,
        attribute_type: super::AttributeType,
    ) -> FilterExpressionFilledOrWaitOperator<T> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let cond = FilterExpressionTypes::AttributeType(placeholder, attribute_type);
        FilterExpressionFilledOrWaitOperator {
            attr: self.attr,
            is_size: self.is_size,
            cond,
            _token: std::marker::PhantomData,
        }
    }

    pub fn contains(
        self,
        value: impl super::IntoAttribute,
    ) -> FilterExpressionFilledOrWaitOperator<T> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let cond = FilterExpressionTypes::Contains(placeholder, value.into_attr());
        FilterExpressionFilledOrWaitOperator {
            attr: self.attr,
            is_size: self.is_size,
            cond,
            _token: std::marker::PhantomData,
        }
    }
}
