pub struct Set<T: super::IntoAttrName> {
    target: T,
    index: Option<usize>,
    value: Option<SetValue<T>>,
}

pub struct SetExpressionFilledWithoutOperation<T: super::IntoAttrName> {
    target: T,
    index: Option<usize>,
    value: SetValue<T>,
    if_not_exists: bool,
}

pub struct SetExpressionFilled<T: super::IntoAttrName> {
    target: T,
    index: Option<usize>,
    value: SetValue<T>,
    if_not_exists: bool,
    operation: SetOperation,
    operand: Operand<T>,
}

enum SetOperation {
    Add,
    Sub,
}

impl std::fmt::Display for SetOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            SetOperation::Add => write!(f, "+"),
            SetOperation::Sub => write!(f, "-"),
        }
    }
}

enum SetValue<T: super::IntoAttrName> {
    Attr(T),
    Value(super::Placeholder, super::AttributeValue),
    // ListAppend(T, List)
}

enum Operand<T: super::IntoAttrName> {
    Attr(T),
    Value(super::Placeholder, super::AttributeValue),
}

impl<T: super::IntoAttrName> Set<T> {
    pub fn new(target: T) -> Self {
        Self {
            target,
            index: None,
            value: None,
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

pub trait SetExpressionBuilder {
    fn build(self) -> (String, super::AttributeNames, super::AttributeValues);
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
            index,
            value,
            if_not_exists,
            operation: SetOperation::Add,
            operand,
        }
    }
}

impl<T: super::IntoAttrName> SetExpressionBuilder for SetExpressionFilledWithoutOperation<T> {
    fn build(self) -> (String, super::AttributeNames, super::AttributeValues) {
        let attr = self.target.into_attr_name();
        let attr_name = format!("#{}", attr.clone());

        let mut names: super::AttributeNames = std::collections::HashMap::new();
        names.insert(attr_name.clone(), attr.clone());
        let mut values: super::AttributeValues = std::collections::HashMap::new();
        match self.value {
            SetValue::Attr(a) => {
                let set_attr = a.into_attr_name();
                let set_attr_name = format!("#{}", set_attr.clone());
                names.insert(set_attr_name.clone(), set_attr.clone());
                let expression = format!("{} = {}", attr_name.clone(), set_attr_name.clone());
                return (expression, names, values);
            }
            SetValue::Value(placeholder, value) => {
                let expression = format!("{} = {}", attr_name.clone(), placeholder);
                values.insert(placeholder.clone(), value);
                return (expression, names, values);
            }
        }
    }
}

impl<T: super::IntoAttrName> SetExpressionBuilder for SetExpressionFilled<T> {
    fn build(self) -> (String, super::AttributeNames, super::AttributeValues) {
        let attr = self.target.into_attr_name();
        let attr_name = format!("#{}", attr.clone());

        let mut names: super::AttributeNames = std::collections::HashMap::new();
        names.insert(attr_name.clone(), attr.clone());
        let mut values: super::AttributeValues = std::collections::HashMap::new();

        let op = format!("{}", self.operation);
        let op_expression = match self.operand {
            Operand::Attr(a) => {
                let operand_attr = a.into_attr_name();
                let operand_attr_name = format!("#{}", operand_attr.clone());
                names.insert(operand_attr_name.clone(), operand_attr.clone());
                format!("{} {}", op, operand_attr_name.clone())
            }
            Operand::Value(placeholder, value) => {
                values.insert(placeholder.clone(), value);
                format!("{} {}", op, placeholder)
            }
        };

        match self.value {
            SetValue::Attr(a) => {
                let set_attr = a.into_attr_name();
                let set_attr_name = format!("#{}", set_attr.clone());
                names.insert(set_attr_name.clone(), set_attr.clone());
                let expression = format!(
                    "{} = {} {}",
                    attr_name.clone(),
                    set_attr_name.clone(),
                    op_expression
                );
                return (expression, names, values);
            }
            SetValue::Value(placeholder, value) => {
                let expression =
                    format!("{} = {} {}", attr_name.clone(), placeholder, op_expression);
                values.insert(placeholder.clone(), value);
                return (expression, names, values);
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
        Id,
        Name,
        Age,
    }

    impl super::super::IntoAttrName for UserAttrNames {
        fn into_attr_name(self) -> String {
            match self {
                UserAttrNames::Id => "id".to_owned(),
                UserAttrNames::Name => "name".to_owned(),
                UserAttrNames::Age => "age".to_owned(),
            }
        }
    }

    #[test]
    fn test_set_value_expression() {
        crate::value_id::reset_value_id();
        let (expression, names, values) = Set::new(UserAttrNames::Name).value("updated!!").build();
        let mut expected_names = std::collections::HashMap::new();
        let mut expected_values = std::collections::HashMap::new();
        expected_names.insert("#name".to_owned(), "name".to_owned());
        expected_values.insert(":value0".to_owned(), "updated!!".into_attr());
        assert_eq!(expression, "#name = :value0".to_owned(),);
        assert_eq!(names, expected_names);
        assert_eq!(values, expected_values);
    }

    #[test]
    fn test_set_attr_expression_with_add_value() {
        crate::value_id::reset_value_id();
        let (expression, names, values) = Set::new(UserAttrNames::Age)
            .attr(UserAttrNames::Age)
            .add_value(10)
            .build();
        let mut expected_names = std::collections::HashMap::new();
        let mut expected_values = std::collections::HashMap::new();
        expected_names.insert("#age".to_owned(), "age".to_owned());
        expected_values.insert(":value0".to_owned(), 10.into_attr());
        assert_eq!(expression, "#age = #age + :value0".to_owned(),);
        assert_eq!(names, expected_names);
        assert_eq!(values, expected_values);
    }
}
