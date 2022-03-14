pub type QueriumString = String;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq)]
pub enum QueriumTypes {
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

#[derive(Debug, Clone)]
pub enum QueriumConjunction {
    And(QueriumString, super::AttributeNames, super::AttributeValues),
}

#[derive(Debug, Clone)]
pub struct Querium<T> {
    pub attr: String,
    pub _token: std::marker::PhantomData<T>,
}

impl<T> Querium<T> {
    pub fn new(attr: String) -> Self {
        Self {
            attr,
            _token: std::marker::PhantomData,
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueriumFilled<T> {
    attr: String,
    cond: QueriumTypes,
    conjunction: QueriumConjunction,
    _token: std::marker::PhantomData<T>,
}

pub trait QueriumBuilder<T> {
    fn build(self) -> (QueriumString, super::AttributeNames, super::AttributeValues);
}

#[derive(Debug, Clone)]
pub struct QueriumFilledOrWaitConjunction<T> {
    attr: String,
    cond: QueriumTypes,
    _token: std::marker::PhantomData<T>,
}

impl<T> QueriumFilledOrWaitConjunction<T> {
    pub fn and(self, cond: impl QueriumBuilder<T>) -> QueriumFilled<T> {
        let (condition_string, attr_names, attr_values) = cond.build();
        QueriumFilled {
            attr: self.attr,
            cond: self.cond,
            conjunction: QueriumConjunction::And(condition_string, attr_names, attr_values),
            _token: self._token,
        }
    }
}

impl<T> QueriumBuilder<T> for QueriumFilled<T> {
    fn build(self) -> (String, super::AttributeNames, super::AttributeValues) {
        let (right_str, right_names, right_values) = match self.conjunction {
            QueriumConjunction::And(s, m, v) => (format!("AND ({})", s), m, v),
        };

        let attr_name = self.attr;
        let mut left_names = super::AttributeNames::new();
        let mut left_values = super::AttributeValues::new();
        left_names.insert(format!("#{}", attr_name), attr_name.clone());

        let left_str = match self.cond {
            QueriumTypes::Eq(placeholder, value) => {
                left_values.insert(placeholder.clone(), value);
                format!("#{} = {}", attr_name, placeholder)
            }
            QueriumTypes::Gt(placeholder, value) => {
                left_values.insert(placeholder.clone(), value);
                format!("#{} > {}", attr_name, placeholder)
            }
            QueriumTypes::Ge(placeholder, value) => {
                left_values.insert(placeholder.clone(), value);
                format!("#{} >= {}", attr_name, placeholder)
            }
            QueriumTypes::Le(placeholder, value) => {
                left_values.insert(placeholder.clone(), value);
                format!("#{} <= {}", attr_name, placeholder)
            }
            QueriumTypes::Lt(placeholder, value) => {
                left_values.insert(placeholder.clone(), value);
                format!("#{} < {}", attr_name, placeholder)
            }
            QueriumTypes::Between(placeholder1, value1, placeholder2, value2) => {
                left_values.insert(placeholder1.clone(), value1);
                left_values.insert(placeholder2.clone(), value2);
                format!(
                    "#{} BETWEEN {} AND {}",
                    attr_name, placeholder1, placeholder2
                )
            }
            QueriumTypes::BeginsWith(placeholder, value) => {
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

impl<T> QueriumBuilder<T> for QueriumFilledOrWaitConjunction<T> {
    fn build(self) -> (String, super::AttributeNames, super::AttributeValues) {
        let attr_name = self.attr;
        let mut attr_names = super::AttributeNames::new();
        let mut attr_values = super::AttributeValues::new();

        attr_names.insert(format!("#{}", attr_name), attr_name.clone());
        match self.cond {
            QueriumTypes::Eq(placeholder, value) => {
                attr_values.insert(placeholder.to_string(), value);
                (
                    format!("#{} = {}", attr_name, placeholder),
                    attr_names,
                    attr_values,
                )
            }
            QueriumTypes::Gt(placeholder, value) => {
                attr_values.insert(placeholder.to_string(), value);
                (
                    format!("#{} > {}", attr_name, placeholder),
                    attr_names,
                    attr_values,
                )
            }
            QueriumTypes::Ge(placeholder, value) => {
                attr_values.insert(placeholder.to_string(), value);
                (
                    format!("#{} >= {}", attr_name, placeholder),
                    attr_names,
                    attr_values,
                )
            }
            QueriumTypes::Le(placeholder, value) => {
                attr_values.insert(placeholder.to_string(), value);
                (
                    format!("#{} <= {}", attr_name, placeholder),
                    attr_names,
                    attr_values,
                )
            }
            QueriumTypes::Lt(placeholder, value) => {
                attr_values.insert(placeholder.to_string(), value);
                (
                    format!("#{} < {}", attr_name, placeholder),
                    attr_names,
                    attr_values,
                )
            }
            QueriumTypes::Between(placeholder1, value1, placeholder2, value2) => {
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
            QueriumTypes::BeginsWith(placeholder, value) => {
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

pub trait QueriumOperand<T>
where
    Self: Sized,
{
    // FIXME: named as `get`, but this method takes the ownership, is that weird??
    fn get_attr(self) -> String;

    fn eq(self, value: impl super::IntoAttribute) -> QueriumFilledOrWaitConjunction<T> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let cond = QueriumTypes::Eq(placeholder, value.into_attr());
        QueriumFilledOrWaitConjunction {
            attr: self.get_attr(),
            cond,
            _token: std::marker::PhantomData,
        }
    }

    fn gt(self, value: impl super::IntoAttribute) -> QueriumFilledOrWaitConjunction<T> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let cond = QueriumTypes::Gt(placeholder, value.into_attr());
        QueriumFilledOrWaitConjunction {
            attr: self.get_attr(),
            cond,
            _token: std::marker::PhantomData,
        }
    }
    fn ge(self, value: impl super::IntoAttribute) -> QueriumFilledOrWaitConjunction<T> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let cond = QueriumTypes::Ge(placeholder, value.into_attr());
        QueriumFilledOrWaitConjunction {
            attr: self.get_attr(),
            cond,
            _token: std::marker::PhantomData,
        }
    }

    fn le(self, value: impl super::IntoAttribute) -> QueriumFilledOrWaitConjunction<T> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let cond = QueriumTypes::Le(placeholder, value.into_attr());
        QueriumFilledOrWaitConjunction {
            attr: self.get_attr(),
            cond,
            _token: std::marker::PhantomData,
        }
    }

    fn lt(self, value: impl super::IntoAttribute) -> QueriumFilledOrWaitConjunction<T> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let cond = QueriumTypes::Lt(placeholder, value.into_attr());
        QueriumFilledOrWaitConjunction {
            attr: self.get_attr(),
            cond,
            _token: std::marker::PhantomData,
        }
    }

    fn between(
        self,
        value1: impl super::IntoAttribute,
        value2: impl super::IntoAttribute,
    ) -> QueriumFilledOrWaitConjunction<T> {
        let placeholder1 = format!(":value{}", super::generate_value_id());
        let placeholder2 = format!(":value{}", super::generate_value_id());
        let cond = QueriumTypes::Between(
            placeholder1,
            value1.into_attr(),
            placeholder2,
            value2.into_attr(),
        );
        QueriumFilledOrWaitConjunction {
            attr: self.get_attr(),
            cond,
            _token: std::marker::PhantomData,
        }
    }

    // We can use `begins_with` only with a range key after specifying an EQ condition for the primary key.
    fn begins_with(self, value: impl super::IntoAttribute) -> QueriumFilledOrWaitConjunction<T> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let cond = QueriumTypes::BeginsWith(placeholder, value.into_attr());
        QueriumFilledOrWaitConjunction {
            attr: self.get_attr(),
            cond,
            _token: std::marker::PhantomData,
        }
    }
}
