use super::*;

pub struct Delete<T: super::IntoAttrName> {
    target: T,
}

pub struct DeleteExpressionFilled<T: super::IntoAttrName> {
    target: T,
    value: (super::Placeholder, super::AttributeValue),
}

impl<T: super::IntoAttrName> Delete<T> {
    pub fn new(target: T) -> Self {
        Self { target }
    }

    pub fn value(self, value: impl super::IntoAttribute) -> DeleteExpressionFilled<T> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let value = (placeholder, value.into_attr());
        let Delete { target } = self;
        DeleteExpressionFilled::<T> { target, value }
    }
}

impl<T: super::IntoAttrName> UpdateExpressionBuilder for DeleteExpressionFilled<T> {
    fn build(self) -> (String, super::AttributeNames, super::AttributeValues) {
        let attr = self.target.into_attr_name();
        let attr_name = format!("#{}", attr);

        let mut names: super::AttributeNames = std::collections::HashMap::new();
        let mut values: super::AttributeValues = std::collections::HashMap::new();
        let (placeholder, value) = self.value;

        // See. https://github.com/raiden-rs/raiden/issues/57
        if value.null.is_some() {
            return ("".to_owned(), names, values);
        }

        names.insert(attr_name.clone(), attr);
        let expression = format!("{} {}", attr_name, placeholder);
        values.insert(placeholder, value);
        (expression, names, values)
    }
}
