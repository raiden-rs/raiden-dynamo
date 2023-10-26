// https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Expressions.OperatorsAndFunctions.html
use super::*;

pub type AttrName = String;

#[derive(Debug, Clone, PartialEq)]
pub enum ConditionFunctionExpression {
    AttributeExists(AttrName),
    AttributeNotExists(AttrName),
    AttributeType(AttrName, super::AttributeType),
    BeginsWith(AttrName, String),
    Contains(AttrName, String),
    Size(AttrName),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConditionComparisonExpression {
    Eq(
        AttrOrPlaceholder,
        Option<super::AttributeValue>,
        AttrOrPlaceholder,
        Option<super::AttributeValue>,
    ),
}

#[derive(Clone, PartialEq)]
pub struct ConditionFilledOrWaitOperator<T: Clone> {
    pub not: bool,
    pub cond: Cond,
    pub _token: std::marker::PhantomData<fn() -> T>,
}

#[derive(Clone, PartialEq)]
pub struct ConditionFilled<T: Clone> {
    pub not: bool,
    pub cond: Cond,
    pub operator: Operator,
    pub _token: std::marker::PhantomData<fn() -> T>,
}

impl<T: Clone> ConditionFilledOrWaitOperator<T> {
    pub fn and(self, cond: impl ConditionBuilder<T>) -> ConditionFilled<T> {
        let (condition_string, attr_names, attr_values) = cond.build();
        ConditionFilled {
            not: self.not,
            cond: self.cond,
            operator: Operator::And(condition_string, attr_names, attr_values),
            _token: self._token,
        }
    }
    pub fn or(self, cond: impl ConditionBuilder<T>) -> ConditionFilled<T> {
        let (condition_string, attr_names, attr_values) = cond.build();
        ConditionFilled {
            not: self.not,
            cond: self.cond,
            operator: Operator::Or(condition_string, attr_names, attr_values),
            _token: self._token,
        }
    }
}

impl<T: Clone> ConditionBuilder<T> for ConditionFilledOrWaitOperator<T> {
    fn build(self) -> (String, super::AttributeNames, super::AttributeValues) {
        if self.not {
            (
                format!("NOT ({})", self.cond.to_string()),
                self.cond.to_attr_names(),
                self.cond.into_attr_values(),
            )
        } else {
            (
                self.cond.to_string(),
                self.cond.to_attr_names(),
                self.cond.into_attr_values(),
            )
        }
    }
}
impl<T: Clone> ConditionBuilder<T> for ConditionFilled<T> {
    fn build(self) -> (String, super::AttributeNames, super::AttributeValues) {
        let (right_str, right_names, right_values) = match self.operator {
            super::condition::Operator::And(s, m, v) => (format!("AND ({s})"), m, v),
            super::condition::Operator::Or(s, m, v) => (format!("OR ({s})"), m, v),
        };
        let left_str = self.cond.to_string();
        let left_names = self.cond.to_attr_names();
        let left_values = self.cond.into_attr_values();
        if self.not {
            (
                format!("NOT ({left_str}) {right_str}"),
                super::merge_map(left_names, right_names),
                super::merge_map(left_values, right_values),
            )
        } else {
            (
                format!("{left_str} {right_str}"),
                super::merge_map(left_names, right_names),
                super::merge_map(left_values, right_values),
            )
        }
    }
}

impl std::string::ToString for ConditionFunctionExpression {
    fn to_string(&self) -> String {
        use crypto::digest::Digest;
        use crypto::md5::Md5;
        match self {
            Self::AttributeExists(path) => format!("attribute_exists(#{path})"),
            Self::AttributeNotExists(path) => format!("attribute_not_exists(#{path})"),
            Self::AttributeType(path, attribute_type) => {
                format!("attribute_type(#{path}, :type{attribute_type})")
            }
            Self::BeginsWith(path, s) => {
                let mut md5 = Md5::new();
                md5.input(s.as_bytes());
                format!("begins_with(#{}, :begins_with_{})", path, md5.result_str())
            }
            Self::Contains(path, s) => {
                let mut md5 = Md5::new();
                md5.input(s.as_bytes());
                format!("contains(#{}, :contains_{})", path, md5.result_str())
            }
            Self::Size(_path) => {
                unimplemented!("Size condition expression is not implemented yet.")
            }
        }
    }
}

impl super::ToAttrNames for ConditionFunctionExpression {
    fn to_attr_names(&self) -> super::AttributeNames {
        let mut m: super::AttributeNames = std::collections::HashMap::new();
        match self {
            Self::Contains(path, _)
            | Self::BeginsWith(path, _)
            | Self::AttributeType(path, _)
            | Self::AttributeExists(path)
            | Self::AttributeNotExists(path) => {
                m.insert(format!("#{path}"), path.clone());
            }
            _ => {}
        }
        m
    }
}

impl super::IntoAttrValues for ConditionFunctionExpression {
    fn into_attr_values(self) -> super::AttributeValues {
        use crypto::digest::Digest;
        use crypto::md5::Md5;
        let mut m: super::AttributeValues = std::collections::HashMap::new();

        #[cfg(any(feature = "rusoto", feature = "rustls"))]
        match self {
            Self::AttributeType(_path, t) => {
                m.insert(
                    format!(":type{t}"),
                    super::AttributeValue {
                        s: Some(t.to_string()),
                        ..super::AttributeValue::default()
                    },
                );
            }
            Self::BeginsWith(_path, s) => {
                let mut md5 = Md5::new();
                md5.input(s.as_bytes());
                m.insert(
                    format!(":begins_with_{}", md5.result_str()),
                    super::AttributeValue {
                        s: Some(s),
                        ..super::AttributeValue::default()
                    },
                );
            }
            Self::Contains(_path, s) => {
                let mut md5 = Md5::new();
                md5.input(s.as_bytes());
                m.insert(
                    format!(":contains_{}", md5.result_str()),
                    super::AttributeValue {
                        s: Some(s),
                        ..super::AttributeValue::default()
                    },
                );
            }
            _ => {}
        }

        #[cfg(feature = "aws-sdk")]
        match self {
            Self::AttributeType(_path, t) => {
                m.insert(format!(":type{t}"), super::AttributeValue::S(t.to_string()));
            }
            Self::BeginsWith(_path, s) => {
                let mut md5 = Md5::new();
                md5.input(s.as_bytes());
                m.insert(
                    format!(":begins_with_{}", md5.result_str()),
                    super::AttributeValue::S(s),
                );
            }
            Self::Contains(_path, s) => {
                let mut md5 = Md5::new();
                md5.input(s.as_bytes());
                m.insert(
                    format!(":contains_{}", md5.result_str()),
                    super::AttributeValue::S(s),
                );
            }
            _ => {}
        }
        m
    }
}

impl std::string::ToString for ConditionComparisonExpression {
    fn to_string(&self) -> String {
        match self {
            Self::Eq(left, _, right, _) => format!("{} = {}", left.to_string(), right.to_string()),
        }
    }
}

impl super::ToAttrNames for ConditionComparisonExpression {
    fn to_attr_names(&self) -> super::AttributeNames {
        let mut m: super::AttributeNames = std::collections::HashMap::new();
        match self {
            Self::Eq(left, _, right, _) => {
                if let AttrOrPlaceholder::Attr(l) = left {
                    m.insert(left.to_string(), l.clone());
                }
                if let AttrOrPlaceholder::Attr(r) = right {
                    m.insert(right.to_string(), r.clone());
                }
            }
        }
        m
    }
}

impl super::IntoAttrValues for ConditionComparisonExpression {
    fn into_attr_values(self) -> super::AttributeValues {
        let mut m: super::AttributeValues = std::collections::HashMap::new();

        match self {
            Self::Eq(left, left_value, right, right_value) => {
                if let Some(left_value) = left_value {
                    m.insert(left.to_string(), left_value);
                }
                if let Some(right_value) = right_value {
                    m.insert(right.to_string(), right_value);
                }
            }
        }
        m
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttrOrPlaceholder {
    Attr(String),
    Placeholder(String),
}

impl std::string::ToString for AttrOrPlaceholder {
    fn to_string(&self) -> String {
        match self {
            Self::Placeholder(p) => format!(":{p}"),
            Self::Attr(a) => format!("#{a}"),
        }
    }
}

pub type ConditionString = String;

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    And(
        ConditionString,
        super::AttributeNames,
        super::AttributeValues,
    ),
    Or(
        ConditionString,
        super::AttributeNames,
        super::AttributeValues,
    ),
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq)]
pub enum Cond {
    Func(ConditionFunctionExpression),
    Cmp(ConditionComparisonExpression),
}

impl std::string::ToString for Cond {
    fn to_string(&self) -> String {
        match self {
            Self::Func(func) => func.to_string(),
            Self::Cmp(cmp) => cmp.to_string(),
        }
    }
}

impl super::ToAttrNames for Cond {
    fn to_attr_names(&self) -> super::AttributeNames {
        match self {
            Self::Func(cond) => cond.to_attr_names(),
            Self::Cmp(cmp) => cmp.to_attr_names(),
        }
    }
}

impl super::IntoAttrValues for Cond {
    fn into_attr_values(self) -> super::AttributeValues {
        match self {
            Self::Func(cond) => cond.into_attr_values(),
            Self::Cmp(cmp) => cmp.into_attr_values(),
        }
    }
}

pub trait ConditionBuilder<T> {
    fn build(
        self,
    ) -> (
        ConditionString,
        super::AttributeNames,
        super::AttributeValues,
    );
}
