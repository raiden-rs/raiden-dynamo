use std::ops::Deref;

use crate::{
    Querium, QueriumBuilder, QueriumConjunction, QueriumFilled, QueriumFilledOrWaitConjunction,
    QueriumOperand, QueriumTypes,
};

#[derive(Debug, Clone)]
pub struct KeyConditionConjunction {
    inner: QueriumConjunction,
}

#[derive(Debug, Clone)]
pub struct KeyConditionTypes {
    inner: QueriumTypes,
}

pub struct KeyCondition<T> {
    inner: Querium<T>,
}

impl<T> KeyCondition<T> {
    fn new(inner: Querium<T>) -> Self {
        Self { inner }
    }

    pub fn from_attr(attr: String) -> Self {
        let inner = Querium::new(attr);
        Self::new(inner)
    }
}

#[derive(Debug, Clone)]
pub struct KeyConditionFilledOrWaitConjunction<T> {
    inner: QueriumFilledOrWaitConjunction<T>,
}

#[derive(Debug, Clone)]
pub struct KeyConditionFilled<T> {
    inner: QueriumFilled<T>,
}

impl Deref for KeyConditionTypes {
    type Target = QueriumTypes;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Deref for KeyConditionConjunction {
    type Target = QueriumConjunction;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> Deref for KeyConditionFilled<T> {
    type Target = QueriumFilled<T>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> Deref for KeyConditionFilledOrWaitConjunction<T> {
    type Target = QueriumFilledOrWaitConjunction<T>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> QueriumBuilder<T> for KeyConditionFilled<T> {
    fn build(self) -> (String, super::AttributeNames, super::AttributeValues) {
        self.inner.build()
    }
}

impl<T> QueriumBuilder<T> for KeyConditionFilledOrWaitConjunction<T> {
    fn build(self) -> (String, super::AttributeNames, super::AttributeValues) {
        self.inner.build()
    }
}

impl<T> QueriumOperand<T> for KeyCondition<T> {
    fn get_attr(self) -> String {
        self.inner.attr
    }
}
