use std::ops::Deref;

use crate::{
    Querium, QueriumBuilder, QueriumConjunction, QueriumFilled, QueriumFilledOrWaitConjunction,
    QueriumOperand, QueriumTypes,
};

pub struct FilterExpressionConjunction {
    inner: QueriumConjunction,
}

pub struct FilterExpression<T> {
    inner: Querium<T>,
}

impl<T> FilterExpression<T> {
    fn new(inner: Querium<T>) -> Self {
        Self { inner }
    }

    pub fn from_attr(attr: String) -> Self {
        let inner = Querium::new(attr);
        Self::new(inner)
    }
}

#[derive(Debug, Clone)]
pub struct FilterExpressionFilledOrWaitConjunction<T> {
    inner: QueriumFilledOrWaitConjunction<T>,
}

#[derive(Debug, Clone)]
pub struct FilterExpressionFilled<T> {
    inner: QueriumFilled<T>,
}

#[derive(Debug, Clone)]
pub struct FilterExpressionTypes {
    inner: QueriumTypes,
}

impl Deref for FilterExpressionTypes {
    type Target = QueriumTypes;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Deref for FilterExpressionConjunction {
    type Target = QueriumConjunction;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> Deref for FilterExpressionFilled<T> {
    type Target = QueriumFilled<T>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> Deref for FilterExpressionFilledOrWaitConjunction<T> {
    type Target = QueriumFilledOrWaitConjunction<T>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> QueriumBuilder<T> for FilterExpressionFilled<T> {
    fn build(self) -> (String, super::AttributeNames, super::AttributeValues) {
        self.inner.build()
    }
}

impl<T> QueriumBuilder<T> for FilterExpressionFilledOrWaitConjunction<T> {
    fn build(self) -> (String, super::AttributeNames, super::AttributeValues) {
        self.inner.build()
    }
}

impl<T> QueriumOperand<T> for FilterExpression<T> {
    fn get_attr(self) -> String {
        self.inner.attr
    }
}
