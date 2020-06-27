pub struct Set<T: super::IntoAttrName> {
    target: T,
    index: Option<usize>,
    value: Option<SetValue<T>>,
}

pub struct SetFilled<T: super::IntoAttrName> {
    target: T,
    index: Option<usize>,
    value: SetValue<T>,
    if_not_exists: bool,
}

enum SetCalculation {
    Add,
    Sub,
}

enum SetValue<T: super::IntoAttrName> {
    Attr(T),
    Value(super::Placeholder, super::AttributeValue),
    // ListAppend(T, List)
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

    pub fn value(self, value: impl super::IntoAttribute) -> SetFilled<T> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let value = SetValue::<T>::Value(placeholder, value.into_attr());
        let Set { target, index, .. } = self;
        SetFilled::<T> {
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

impl<T: super::IntoAttrName> SetFilled<T> {
    pub fn if_not_exists(mut self) -> SetFilled<T> {
        self.if_not_exists = true;
        self
    }
}

impl<T: super::IntoAttrName> SetExpressionBuilder for SetFilled<T> {
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
    }

    impl super::super::IntoAttrName for UserAttrNames {
        fn into_attr_name(self) -> String {
            match self {
                UserAttrNames::Id => "id".to_owned(),
                UserAttrNames::Name => "name".to_owned(),
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
}
