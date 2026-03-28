pub type KeyConditionString = String;

#[derive(Debug, Clone)]
pub enum KeyConditionOperator {
    And(
        KeyConditionString,
        super::AttributeNames,
        super::AttributeValues,
    ),
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq)]
pub enum KeyConditionTypes {
    Eq(super::Placeholder, super::AttributeValue),
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

pub trait KeyConditionBuilder<T, U> {
    fn build(
        self,
    ) -> (
        KeyConditionString,
        super::AttributeNames,
        super::AttributeValues,
    );
}

pub trait SupportsEqCondition {}
pub trait SupportsRangeCondition {}

#[derive(Debug, Clone)]
pub struct KeyCondition<T, U = T> {
    pub attr: String,
    pub _token: std::marker::PhantomData<fn() -> T>,
    pub _next_token: std::marker::PhantomData<fn() -> U>,
}

#[derive(Debug, Clone)]
pub struct KeyConditionFilledOrWaitOperator<T, U> {
    attr: String,
    cond: KeyConditionTypes,
    _token: std::marker::PhantomData<fn() -> T>,
    _next_token: std::marker::PhantomData<fn() -> U>,
}

#[derive(Debug, Clone)]
pub struct KeyConditionFilled<T, U> {
    attr: String,
    cond: KeyConditionTypes,
    operators: Vec<KeyConditionOperator>,
    _token: std::marker::PhantomData<fn() -> T>,
    _next_token: std::marker::PhantomData<fn() -> U>,
}

impl<T, U> KeyConditionFilledOrWaitOperator<T, U> {
    pub fn and<V>(self, cond: impl KeyConditionBuilder<U, V>) -> KeyConditionFilled<T, V> {
        let (condition_string, attr_names, attr_values) = cond.build();
        KeyConditionFilled {
            attr: self.attr,
            cond: self.cond,
            operators: vec![KeyConditionOperator::And(
                condition_string,
                attr_names,
                attr_values,
            )],
            _token: self._token,
            _next_token: std::marker::PhantomData,
        }
    }
}

impl<T, U> KeyConditionFilled<T, U> {
    pub fn and<V>(mut self, cond: impl KeyConditionBuilder<U, V>) -> KeyConditionFilled<T, V> {
        let (condition_string, attr_names, attr_values) = cond.build();
        self.operators.push(KeyConditionOperator::And(
            condition_string,
            attr_names,
            attr_values,
        ));
        KeyConditionFilled {
            attr: self.attr,
            cond: self.cond,
            operators: self.operators,
            _token: self._token,
            _next_token: std::marker::PhantomData,
        }
    }
}

impl<T, U> KeyConditionBuilder<T, U> for KeyConditionFilledOrWaitOperator<T, U> {
    fn build(self) -> (String, super::AttributeNames, super::AttributeValues) {
        let attr_name = self.attr;
        let mut attr_names: super::AttributeNames = std::collections::HashMap::new();
        let mut attr_values: super::AttributeValues = std::collections::HashMap::new();

        attr_names.insert(format!("#{attr_name}"), attr_name.clone());
        match self.cond {
            super::key_condition::KeyConditionTypes::Eq(placeholder, value) => {
                attr_values.insert(placeholder.to_string(), value);
                (
                    format!("#{attr_name} = {placeholder}"),
                    attr_names,
                    attr_values,
                )
            }
            super::key_condition::KeyConditionTypes::Gt(placeholder, value) => {
                attr_values.insert(placeholder.to_string(), value);
                (
                    format!("#{attr_name} > {placeholder}"),
                    attr_names,
                    attr_values,
                )
            }
            super::key_condition::KeyConditionTypes::Ge(placeholder, value) => {
                attr_values.insert(placeholder.to_string(), value);
                (
                    format!("#{attr_name} >= {placeholder}"),
                    attr_names,
                    attr_values,
                )
            }
            super::key_condition::KeyConditionTypes::Le(placeholder, value) => {
                attr_values.insert(placeholder.to_string(), value);
                (
                    format!("#{attr_name} <= {placeholder}"),
                    attr_names,
                    attr_values,
                )
            }
            super::key_condition::KeyConditionTypes::Lt(placeholder, value) => {
                attr_values.insert(placeholder.to_string(), value);
                (
                    format!("#{attr_name} < {placeholder}"),
                    attr_names,
                    attr_values,
                )
            }
            super::key_condition::KeyConditionTypes::Between(
                placeholder1,
                value1,
                placeholder2,
                value2,
            ) => {
                attr_values.insert(placeholder1.to_string(), value1);
                attr_values.insert(placeholder2.to_string(), value2);
                (
                    format!("#{attr_name} BETWEEN {placeholder1} AND {placeholder2}"),
                    attr_names,
                    attr_values,
                )
            }
            super::key_condition::KeyConditionTypes::BeginsWith(placeholder, value) => {
                attr_values.insert(placeholder.to_string(), value);
                (
                    format!("begins_with(#{attr_name}, {placeholder})"),
                    attr_names,
                    attr_values,
                )
            }
        }
    }
}

impl<T, U> KeyConditionBuilder<T, U> for KeyConditionFilled<T, U> {
    fn build(self) -> (String, super::AttributeNames, super::AttributeValues) {
        let attr_name = self.attr;
        let mut left_names: super::AttributeNames = std::collections::HashMap::new();
        let mut left_values: super::AttributeValues = std::collections::HashMap::new();
        left_names.insert(format!("#{attr_name}"), attr_name.clone());

        let left_str = match self.cond {
            super::key_condition::KeyConditionTypes::Eq(placeholder, value) => {
                left_values.insert(placeholder.clone(), value);
                format!("#{attr_name} = {placeholder}")
            }
            super::key_condition::KeyConditionTypes::Gt(placeholder, value) => {
                left_values.insert(placeholder.clone(), value);
                format!("#{attr_name} > {placeholder}")
            }
            super::key_condition::KeyConditionTypes::Ge(placeholder, value) => {
                left_values.insert(placeholder.clone(), value);
                format!("#{attr_name} >= {placeholder}")
            }
            super::key_condition::KeyConditionTypes::Le(placeholder, value) => {
                left_values.insert(placeholder.clone(), value);
                format!("#{attr_name} <= {placeholder}")
            }
            super::key_condition::KeyConditionTypes::Lt(placeholder, value) => {
                left_values.insert(placeholder.clone(), value);
                format!("#{attr_name} < {placeholder}")
            }
            super::key_condition::KeyConditionTypes::Between(
                placeholder1,
                value1,
                placeholder2,
                value2,
            ) => {
                left_values.insert(placeholder1.clone(), value1);
                left_values.insert(placeholder2.clone(), value2);
                format!("#{attr_name} BETWEEN {placeholder1} AND {placeholder2}")
            }
            super::key_condition::KeyConditionTypes::BeginsWith(placeholder, value) => {
                left_values.insert(placeholder.clone(), value);
                format!("begins_with(#{attr_name}, {placeholder})")
            }
        };

        let mut condition_strings = vec![left_str];
        let mut merged_names = left_names;
        let mut merged_values = left_values;

        for operator in self.operators {
            match operator {
                super::key_condition::KeyConditionOperator::And(s, m, v) => {
                    condition_strings.push(format!("AND ({s})"));
                    merged_names = super::merge_map(merged_names, m);
                    merged_values = super::merge_map(merged_values, v);
                }
            }
        }

        (condition_strings.join(" "), merged_names, merged_values)
    }
}

impl<T, U> KeyCondition<T, U>
where
    T: SupportsEqCondition,
{
    pub fn eq(self, value: impl super::IntoAttribute) -> KeyConditionFilledOrWaitOperator<T, U> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let cond = super::key_condition::KeyConditionTypes::Eq(placeholder, value.into_attr());
        KeyConditionFilledOrWaitOperator {
            attr: self.attr,
            cond,
            _token: std::marker::PhantomData,
            _next_token: std::marker::PhantomData,
        }
    }
}

impl<T, U> KeyCondition<T, U>
where
    T: SupportsRangeCondition,
{
    pub fn gt(self, value: impl super::IntoAttribute) -> KeyConditionFilledOrWaitOperator<T, U> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let cond = super::key_condition::KeyConditionTypes::Gt(placeholder, value.into_attr());
        KeyConditionFilledOrWaitOperator {
            attr: self.attr,
            cond,
            _token: std::marker::PhantomData,
            _next_token: std::marker::PhantomData,
        }
    }
    pub fn ge(self, value: impl super::IntoAttribute) -> KeyConditionFilledOrWaitOperator<T, U> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let cond = super::key_condition::KeyConditionTypes::Ge(placeholder, value.into_attr());
        KeyConditionFilledOrWaitOperator {
            attr: self.attr,
            cond,
            _token: std::marker::PhantomData,
            _next_token: std::marker::PhantomData,
        }
    }

    pub fn le(self, value: impl super::IntoAttribute) -> KeyConditionFilledOrWaitOperator<T, U> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let cond = super::key_condition::KeyConditionTypes::Le(placeholder, value.into_attr());
        KeyConditionFilledOrWaitOperator {
            attr: self.attr,
            cond,
            _token: std::marker::PhantomData,
            _next_token: std::marker::PhantomData,
        }
    }

    pub fn lt(self, value: impl super::IntoAttribute) -> KeyConditionFilledOrWaitOperator<T, U> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let cond = super::key_condition::KeyConditionTypes::Lt(placeholder, value.into_attr());
        KeyConditionFilledOrWaitOperator {
            attr: self.attr,
            cond,
            _token: std::marker::PhantomData,
            _next_token: std::marker::PhantomData,
        }
    }

    pub fn between(
        self,
        value1: impl super::IntoAttribute,
        value2: impl super::IntoAttribute,
    ) -> KeyConditionFilledOrWaitOperator<T, U> {
        let placeholder1 = format!(":value{}", super::generate_value_id());
        let placeholder2 = format!(":value{}", super::generate_value_id());
        let cond = super::key_condition::KeyConditionTypes::Between(
            placeholder1,
            value1.into_attr(),
            placeholder2,
            value2.into_attr(),
        );
        KeyConditionFilledOrWaitOperator {
            attr: self.attr,
            cond,
            _token: std::marker::PhantomData,
            _next_token: std::marker::PhantomData,
        }
    }

    // We can use `begins_with` only with a range key after specifying an EQ condition for the primary key.
    pub fn begins_with(
        self,
        value: impl super::IntoAttribute,
    ) -> KeyConditionFilledOrWaitOperator<T, U> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let cond =
            super::key_condition::KeyConditionTypes::BeginsWith(placeholder, value.into_attr());
        KeyConditionFilledOrWaitOperator {
            attr: self.attr,
            cond,
            _token: std::marker::PhantomData,
            _next_token: std::marker::PhantomData,
        }
    }
}
