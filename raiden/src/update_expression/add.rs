use super::*;

pub struct Add<T: super::IntoAttrName> {
    target: T,
}

pub struct AddExpressionFilled<T: super::IntoAttrName> {
    target: T,
    value: (super::Placeholder, super::AttributeValue),
}

impl<T: super::IntoAttrName> Add<T> {
    pub fn new(target: T) -> Self {
        Self { target }
    }

    pub fn value(self, value: impl super::IntoAttribute) -> AddExpressionFilled<T> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let value = (placeholder, value.into_attr());
        let Add { target } = self;
        AddExpressionFilled::<T> { target, value }
    }
}

impl<T: super::IntoAttrName> UpdateAddExpressionBuilder for AddExpressionFilled<T> {
    fn build(self) -> (String, super::AttributeNames, super::AttributeValues) {
        let attr = self.target.into_attr_name();
        let attr_name = format!("#{attr}");

        let mut names: super::AttributeNames = std::collections::HashMap::new();
        let mut values: super::AttributeValues = std::collections::HashMap::new();
        let (placeholder, value) = self.value;

        // See. https://github.com/raiden-rs/raiden/issues/57
        //      https://github.com/raiden-rs/raiden/issues/58
        #[cfg(any(feature = "rusoto", feature = "rustls"))]
        let is_null = value.null.is_some();
        #[cfg(feature = "aws-sdk")]
        let is_null = value.is_null();

        if is_null || crate::is_attr_value_empty(&value) {
            return ("".to_owned(), names, values);
        }

        names.insert(attr_name.clone(), attr);
        let expression = format!("{attr_name} {placeholder}");
        values.insert(placeholder, value);
        (expression, names, values)
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
        Age,
    }

    impl super::super::IntoAttrName for UserAttrNames {
        fn into_attr_name(self) -> String {
            match self {
                UserAttrNames::Age => "age".to_owned(),
            }
        }
    }

    #[test]
    fn test_add_value_expression() {
        crate::value_id::reset_value_id();
        let (expression, names, values) = Add::new(UserAttrNames::Age).value(42).build();
        let mut expected_names = std::collections::HashMap::new();
        let mut expected_values = std::collections::HashMap::new();
        expected_names.insert("#age".to_owned(), "age".to_owned());
        expected_values.insert(":value0".to_owned(), 42.into_attr());
        assert_eq!(expression, "#age :value0".to_owned(),);
        assert_eq!(names, expected_names);
        assert_eq!(values, expected_values);
    }
}
