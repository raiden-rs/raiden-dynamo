pub type KeyConditionString = String;

pub enum KeyConditionConjunction {
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

pub trait KeyConditionBuilder<T> {
    fn build(
        self,
    ) -> (
        KeyConditionString,
        super::AttributeNames,
        super::AttributeValues,
    );
}

#[derive(Debug, Clone)]
pub struct KeyCondition<T> {
    pub attr: String,
    pub _token: std::marker::PhantomData<T>,
}

pub struct KeyConditionFilledOrWaitConjunction<T> {
    attr: String,
    cond: KeyConditionTypes,
    _token: std::marker::PhantomData<T>,
}

pub struct KeyConditionFilled<T> {
    attr: String,
    cond: KeyConditionTypes,
    conjunction: KeyConditionConjunction,
    _token: std::marker::PhantomData<T>,
}

impl<T> KeyConditionFilledOrWaitConjunction<T> {
    pub fn and(self, cond: impl KeyConditionBuilder<T>) -> KeyConditionFilled<T> {
        let (condition_string, attr_names, attr_values) = cond.build();
        KeyConditionFilled {
            attr: self.attr,
            cond: self.cond,
            conjunction: KeyConditionConjunction::And(condition_string, attr_names, attr_values),
            _token: self._token,
        }
    }
}

impl<T> KeyConditionBuilder<T> for KeyConditionFilledOrWaitConjunction<T> {
    fn build(self) -> (String, super::AttributeNames, super::AttributeValues) {
        let attr_name = self.attr;
        let mut attr_names: super::AttributeNames = std::collections::HashMap::new();
        let mut attr_values: super::AttributeValues = std::collections::HashMap::new();

        attr_names.insert(format!("#{}", attr_name), attr_name.clone());
        match self.cond {
            super::key_condition::KeyConditionTypes::Eq(placeholder, value) => {
                attr_values.insert(placeholder.to_string(), value);
                (
                    format!("#{} = {}", attr_name, placeholder),
                    attr_names,
                    attr_values,
                )
            }
            super::key_condition::KeyConditionTypes::Gt(placeholder, value) => {
                attr_values.insert(placeholder.to_string(), value);
                (
                    format!("#{} > {}", attr_name, placeholder),
                    attr_names,
                    attr_values,
                )
            }
            super::key_condition::KeyConditionTypes::Ge(placeholder, value) => {
                attr_values.insert(placeholder.to_string(), value);
                (
                    format!("#{} >= {}", attr_name, placeholder),
                    attr_names,
                    attr_values,
                )
            }
            super::key_condition::KeyConditionTypes::Le(placeholder, value) => {
                attr_values.insert(placeholder.to_string(), value);
                (
                    format!("#{} <= {}", attr_name, placeholder),
                    attr_names,
                    attr_values,
                )
            }
            super::key_condition::KeyConditionTypes::Lt(placeholder, value) => {
                attr_values.insert(placeholder.to_string(), value);
                (
                    format!("#{} < {}", attr_name, placeholder),
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
                    format!(
                        "#{} BETWEEN {} AND {}",
                        attr_name, placeholder1, placeholder2
                    ),
                    attr_names,
                    attr_values,
                )
            }
            super::key_condition::KeyConditionTypes::BeginsWith(placeholder, value) => {
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

impl<T> KeyConditionBuilder<T> for KeyConditionFilled<T> {
    fn build(self) -> (String, super::AttributeNames, super::AttributeValues) {
        let (right_str, right_names, right_values) = match self.conjunction {
            super::key_condition::KeyConditionConjunction::And(s, m, v) => {
                (format!("AND ({})", s), m, v)
            }
        };

        let attr_name = self.attr;
        let mut left_names: super::AttributeNames = std::collections::HashMap::new();
        let mut left_values: super::AttributeValues = std::collections::HashMap::new();
        left_names.insert(format!("#{}", attr_name), attr_name.clone());

        let left_str = match self.cond {
            super::key_condition::KeyConditionTypes::Eq(placeholder, value) => {
                left_values.insert(placeholder.clone(), value);
                format!("#{} = {}", attr_name, placeholder)
            }
            super::key_condition::KeyConditionTypes::Gt(placeholder, value) => {
                left_values.insert(placeholder.clone(), value);
                format!("#{} > {}", attr_name, placeholder)
            }
            super::key_condition::KeyConditionTypes::Ge(placeholder, value) => {
                left_values.insert(placeholder.clone(), value);
                format!("#{} >= {}", attr_name, placeholder)
            }
            super::key_condition::KeyConditionTypes::Le(placeholder, value) => {
                left_values.insert(placeholder.clone(), value);
                format!("#{} <= {}", attr_name, placeholder)
            }
            super::key_condition::KeyConditionTypes::Lt(placeholder, value) => {
                left_values.insert(placeholder.clone(), value);
                format!("#{} < {}", attr_name, placeholder)
            }
            super::key_condition::KeyConditionTypes::Between(
                placeholder1,
                value1,
                placeholder2,
                value2,
            ) => {
                left_values.insert(placeholder1.clone(), value1);
                left_values.insert(placeholder2.clone(), value2);
                format!(
                    "#{} BETWEEN {} AND {}",
                    attr_name, placeholder1, placeholder2
                )
            }
            super::key_condition::KeyConditionTypes::BeginsWith(placeholder, value) => {
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

impl<T> KeyCondition<T> {
    pub fn eq(self, value: impl super::IntoAttribute) -> KeyConditionFilledOrWaitConjunction<T> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let cond = super::key_condition::KeyConditionTypes::Eq(placeholder, value.into_attr());
        KeyConditionFilledOrWaitConjunction {
            attr: self.attr,
            cond,
            _token: std::marker::PhantomData,
        }
    }

    pub fn gt(self, value: impl super::IntoAttribute) -> KeyConditionFilledOrWaitConjunction<T> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let cond = super::key_condition::KeyConditionTypes::Gt(placeholder, value.into_attr());
        KeyConditionFilledOrWaitConjunction {
            attr: self.attr,
            cond,
            _token: std::marker::PhantomData,
        }
    }
    pub fn ge(self, value: impl super::IntoAttribute) -> KeyConditionFilledOrWaitConjunction<T> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let cond = super::key_condition::KeyConditionTypes::Ge(placeholder, value.into_attr());
        KeyConditionFilledOrWaitConjunction {
            attr: self.attr,
            cond,
            _token: std::marker::PhantomData,
        }
    }

    pub fn le(self, value: impl super::IntoAttribute) -> KeyConditionFilledOrWaitConjunction<T> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let cond = super::key_condition::KeyConditionTypes::Le(placeholder, value.into_attr());
        KeyConditionFilledOrWaitConjunction {
            attr: self.attr,
            cond,
            _token: std::marker::PhantomData,
        }
    }

    pub fn lt(self, value: impl super::IntoAttribute) -> KeyConditionFilledOrWaitConjunction<T> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let cond = super::key_condition::KeyConditionTypes::Lt(placeholder, value.into_attr());
        KeyConditionFilledOrWaitConjunction {
            attr: self.attr,
            cond,
            _token: std::marker::PhantomData,
        }
    }

    pub fn between(
        self,
        value1: impl super::IntoAttribute,
        value2: impl super::IntoAttribute,
    ) -> KeyConditionFilledOrWaitConjunction<T> {
        let placeholder1 = format!(":value{}", super::generate_value_id());
        let placeholder2 = format!(":value{}", super::generate_value_id());
        let cond = super::key_condition::KeyConditionTypes::Between(
            placeholder1,
            value1.into_attr(),
            placeholder2,
            value2.into_attr(),
        );
        KeyConditionFilledOrWaitConjunction {
            attr: self.attr,
            cond,
            _token: std::marker::PhantomData,
        }
    }

    pub fn begins_with(
        self,
        value: impl super::IntoAttribute,
    ) -> KeyConditionFilledOrWaitConjunction<T> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let cond =
            super::key_condition::KeyConditionTypes::BeginsWith(placeholder, value.into_attr());
        KeyConditionFilledOrWaitConjunction {
            attr: self.attr,
            cond,
            _token: std::marker::PhantomData,
        }
    }
}
