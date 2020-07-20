use super::*;

pub struct Set<T: super::IntoAttrName> {
    target: T,
    index: Option<usize>,
    _value: Option<SetValue<T>>,
}

pub struct SetExpressionFilledWithoutOperation<T: super::IntoAttrName> {
    target: T,
    index: Option<usize>,
    value: SetValue<T>,
    if_not_exists: bool,
}

pub struct SetExpressionFilled<T: super::IntoAttrName> {
    target: T,
    _index: Option<usize>,
    value: SetValue<T>,
    _if_not_exists: bool,
    operation: SetOperation,
    operand: Operand<T>,
}

enum SetOperation {
    Add,
    _Sub,
}

impl std::fmt::Display for SetOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            SetOperation::Add => write!(f, "+"),
            SetOperation::_Sub => write!(f, "-"),
        }
    }
}

#[allow(clippy::large_enum_variant)]
enum SetValue<T: super::IntoAttrName> {
    Attr(T),
    Value(super::Placeholder, super::AttributeValue),
    // ListAppend(T, List)
}

#[allow(clippy::large_enum_variant)]
enum Operand<T: super::IntoAttrName> {
    _Attr(T),
    Value(super::Placeholder, super::AttributeValue),
}

impl<T: super::IntoAttrName> Set<T> {
    pub fn new(target: T) -> Self {
        Self {
            target,
            index: None,
            _value: None,
        }
    }

    // For LIST/SET
    pub fn index(mut self, index: usize) -> Self {
        self.index = Some(index);
        self
    }

    pub fn value(self, value: impl super::IntoAttribute) -> SetExpressionFilledWithoutOperation<T> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let value = SetValue::<T>::Value(placeholder, value.into_attr());
        let Set { target, index, .. } = self;
        SetExpressionFilledWithoutOperation::<T> {
            target,
            index,
            value,
            if_not_exists: false,
        }
    }

    pub fn attr(self, attr: T) -> SetExpressionFilledWithoutOperation<T> {
        let value = SetValue::<T>::Attr(attr);
        let Set { target, index, .. } = self;
        SetExpressionFilledWithoutOperation::<T> {
            target,
            index,
            value,
            if_not_exists: false,
        }
    }
}

impl<T: super::IntoAttrName> SetExpressionFilledWithoutOperation<T> {
    pub fn if_not_exists(mut self) -> SetExpressionFilledWithoutOperation<T> {
        self.if_not_exists = true;
        self
    }

    pub fn add_value(self, value: impl super::IntoAttribute) -> SetExpressionFilled<T> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let operand = Operand::<T>::Value(placeholder, value.into_attr());
        let SetExpressionFilledWithoutOperation {
            target,
            index,
            value,
            if_not_exists,
        } = self;
        SetExpressionFilled::<T> {
            target,
            _index: index,
            value,
            _if_not_exists: if_not_exists,
            operation: SetOperation::Add,
            operand,
        }
    }
}

impl<T: super::IntoAttrName> UpdateSetExpressionBuilder for SetExpressionFilledWithoutOperation<T> {
    fn build(self) -> SetOrRemove {
        let attr = self.target.into_attr_name();
        let attr_name = format!("#{}", attr);

        let mut names: super::AttributeNames = std::collections::HashMap::new();
        names.insert(attr_name.clone(), attr);
        let mut values: super::AttributeValues = std::collections::HashMap::new();
        match self.value {
            SetValue::Attr(a) => {
                let set_attr = a.into_attr_name();
                let set_attr_name = format!("#{}", set_attr);
                let expression = format!("{} = {}", attr_name, set_attr_name);
                names.insert(set_attr_name, set_attr);
                SetOrRemove::Set(expression, names, values)
            }
            SetValue::Value(placeholder, value) => {
                // See. https://github.com/raiden-rs/raiden/issues/57
                //      https://github.com/raiden-rs/raiden/issues/58
                if value.null.is_some() || value == AttributeValue::default() {
                    // Use remove instead of set
                    return SetOrRemove::Remove(attr_name, names);
                }
                let expression = format!("{} = {}", attr_name, placeholder);
                values.insert(placeholder, value);
                SetOrRemove::Set(expression, names, values)
            }
        }
    }
}

impl<T: super::IntoAttrName> UpdateSetExpressionBuilder for SetExpressionFilled<T> {
    fn build(self) -> SetOrRemove {
        let attr = self.target.into_attr_name();
        let attr_name = format!("#{}", attr);

        let mut names: super::AttributeNames = std::collections::HashMap::new();
        names.insert(attr_name.clone(), attr);
        let mut values: super::AttributeValues = std::collections::HashMap::new();

        let op = format!("{}", self.operation);
        let op_expression = match self.operand {
            Operand::_Attr(a) => {
                let operand_attr = a.into_attr_name();
                let operand_attr_name = format!("#{}", operand_attr);
                let val = format!("{} {}", op, operand_attr_name);
                names.insert(operand_attr_name, operand_attr);
                val
            }
            Operand::Value(placeholder, value) => {
                let val = format!("{} {}", op, placeholder);
                values.insert(placeholder, value);
                val
            }
        };

        match self.value {
            SetValue::Attr(a) => {
                let set_attr = a.into_attr_name();
                let set_attr_name = format!("#{}", set_attr);
                let expression = format!("{} = {} {}", attr_name, set_attr_name, op_expression);
                names.insert(set_attr_name, set_attr);
                SetOrRemove::Set(expression, names, values)
            }
            SetValue::Value(placeholder, value) => {
                let expression = format!("{} = {} {}", attr_name, placeholder, op_expression);
                values.insert(placeholder, value);
                SetOrRemove::Set(expression, names, values)
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::*;

    #[cfg(test)]
    use pretty_assertions::assert_eq;

    #[derive(Debug, Clone, Copy, PartialEq)]
    enum UserAttrNames {
        Name,
        Age,
    }

    impl super::super::IntoAttrName for UserAttrNames {
        fn into_attr_name(self) -> String {
            match self {
                UserAttrNames::Name => "name".to_owned(),
                UserAttrNames::Age => "age".to_owned(),
            }
        }
    }

    #[test]
    fn test_set_value_expression() {
        crate::value_id::reset_value_id();
        if let SetOrRemove::Set(expression, names, values) =
            Set::new(UserAttrNames::Name).value("updated!!").build()
        {
            let mut expected_names = std::collections::HashMap::new();
            let mut expected_values = std::collections::HashMap::new();
            expected_names.insert("#name".to_owned(), "name".to_owned());
            expected_values.insert(":value0".to_owned(), "updated!!".into_attr());
            assert_eq!(expression, "#name = :value0".to_owned(),);
            assert_eq!(names, expected_names);
            assert_eq!(values, expected_values);
            return;
        }
        panic!("should not pass");
    }

    #[test]
    fn test_set_attr_expression_with_add_value() {
        crate::value_id::reset_value_id();
        if let SetOrRemove::Set(expression, names, values) = Set::new(UserAttrNames::Age)
            .attr(UserAttrNames::Age)
            .add_value(10)
            .build()
        {
            let mut expected_names = std::collections::HashMap::new();
            let mut expected_values = std::collections::HashMap::new();
            expected_names.insert("#age".to_owned(), "age".to_owned());
            expected_values.insert(":value0".to_owned(), 10.into_attr());
            assert_eq!(expression, "#age = #age + :value0".to_owned(),);
            assert_eq!(names, expected_names);
            assert_eq!(values, expected_values);
            return;
        }
        panic!("should not pass");
    }
}
