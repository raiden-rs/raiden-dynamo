// https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Expressions.OperatorsAndFunctions.html
use super::*;

pub type AttrName = super::AttrPath;

#[derive(Debug, Clone, PartialEq)]
pub enum ConditionFunctionExpression {
    AttributeExists(AttrName),
    AttributeNotExists(AttrName),
    AttributeType(AttrName, super::AttributeType),
    BeginsWith(AttrName, String),
    Contains(AttrName, String),
    ContainsValue(AttrName, super::Placeholder, Box<super::AttributeValue>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionComparisonOperator {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl std::fmt::Display for ConditionComparisonOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let operator = match self {
            Self::Eq => "=",
            Self::Ne => "<>",
            Self::Lt => "<",
            Self::Le => "<=",
            Self::Gt => ">",
            Self::Ge => ">=",
        };
        write!(f, "{operator}")
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum ConditionComparisonExpression {
    Eq(
        AttrOrPlaceholder,
        Option<super::AttributeValue>,
        AttrOrPlaceholder,
        Option<super::AttributeValue>,
    ),
    Compare(
        AttrOrPlaceholder,
        Option<super::AttributeValue>,
        ConditionComparisonOperator,
        AttrOrPlaceholder,
        Option<super::AttributeValue>,
    ),
    Between(
        AttrName,
        super::Placeholder,
        super::AttributeValue,
        super::Placeholder,
        super::AttributeValue,
    ),
    In(AttrName, Vec<(super::Placeholder, super::AttributeValue)>),
    Size(
        AttrName,
        ConditionComparisonOperator,
        super::Placeholder,
        super::AttributeValue,
    ),
}

#[derive(Clone, PartialEq)]
pub struct ConditionSize<T: Clone> {
    pub not: bool,
    pub attr: AttrName,
    pub _token: std::marker::PhantomData<fn() -> T>,
}

impl<T: Clone> ConditionSize<T> {
    fn compare(
        self,
        operator: ConditionComparisonOperator,
        value: impl super::IntoNumberAttribute,
    ) -> ConditionFilledOrWaitOperator<T> {
        let placeholder = format!(":value{}", super::generate_value_id());
        let cond = Cond::Cmp(ConditionComparisonExpression::Size(
            self.attr,
            operator,
            placeholder,
            value.into_number_attr(),
        ));
        ConditionFilledOrWaitOperator {
            not: self.not,
            cond,
            _token: self._token,
        }
    }

    pub fn eq(self, value: impl super::IntoNumberAttribute) -> ConditionFilledOrWaitOperator<T> {
        self.compare(ConditionComparisonOperator::Eq, value)
    }

    pub fn ne(self, value: impl super::IntoNumberAttribute) -> ConditionFilledOrWaitOperator<T> {
        self.compare(ConditionComparisonOperator::Ne, value)
    }

    pub fn lt(self, value: impl super::IntoNumberAttribute) -> ConditionFilledOrWaitOperator<T> {
        self.compare(ConditionComparisonOperator::Lt, value)
    }

    pub fn le(self, value: impl super::IntoNumberAttribute) -> ConditionFilledOrWaitOperator<T> {
        self.compare(ConditionComparisonOperator::Le, value)
    }

    pub fn gt(self, value: impl super::IntoNumberAttribute) -> ConditionFilledOrWaitOperator<T> {
        self.compare(ConditionComparisonOperator::Gt, value)
    }

    pub fn ge(self, value: impl super::IntoNumberAttribute) -> ConditionFilledOrWaitOperator<T> {
        self.compare(ConditionComparisonOperator::Ge, value)
    }
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
                format!("NOT ({})", self.cond),
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

impl std::fmt::Display for ConditionFunctionExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use md5::{Digest, Md5};

        match self {
            Self::AttributeExists(path) => write!(f, "attribute_exists({path})"),
            Self::AttributeNotExists(path) => write!(f, "attribute_not_exists({path})"),
            Self::AttributeType(path, attribute_type) => {
                write!(f, "attribute_type({path}, :type{attribute_type})")
            }
            Self::BeginsWith(path, s) => {
                let mut hasher = Md5::new();
                hasher.update(s.as_bytes());
                write!(
                    f,
                    "begins_with({path}, :begins_with_{:x})",
                    hasher.finalize()
                )
            }
            Self::Contains(path, s) => {
                let mut hasher = Md5::new();
                hasher.update(s.as_bytes());
                write!(f, "contains({path}, :contains_{:x})", hasher.finalize())
            }
            Self::ContainsValue(path, placeholder, _) => {
                write!(f, "contains({path}, {placeholder})")
            }
        }
    }
}

impl super::ToAttrNames for ConditionFunctionExpression {
    fn to_attr_names(&self) -> super::AttributeNames {
        match self {
            Self::Contains(path, _)
            | Self::ContainsValue(path, _, _)
            | Self::BeginsWith(path, _)
            | Self::AttributeType(path, _)
            | Self::AttributeExists(path)
            | Self::AttributeNotExists(path) => path.attribute_names(),
        }
    }
}

impl super::IntoAttrValues for ConditionFunctionExpression {
    fn into_attr_values(self) -> super::AttributeValues {
        use md5::{Digest, Md5};

        let mut m: super::AttributeValues = std::collections::HashMap::new();

        #[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
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
                let mut hasher = Md5::new();
                hasher.update(s.as_bytes());
                m.insert(
                    format!(":begins_with_{:x}", hasher.finalize()),
                    super::AttributeValue {
                        s: Some(s),
                        ..super::AttributeValue::default()
                    },
                );
            }
            Self::Contains(_path, s) => {
                let mut hasher = Md5::new();
                hasher.update(s.as_bytes());
                m.insert(
                    format!(":contains_{:x}", hasher.finalize()),
                    super::AttributeValue {
                        s: Some(s),
                        ..super::AttributeValue::default()
                    },
                );
            }
            Self::ContainsValue(_path, placeholder, value) => {
                m.insert(placeholder, *value);
            }
            _ => {}
        }

        #[cfg(feature = "aws-sdk")]
        match self {
            Self::AttributeType(_path, t) => {
                m.insert(format!(":type{t}"), super::AttributeValue::S(t.to_string()));
            }
            Self::BeginsWith(_path, s) => {
                let mut hasher = Md5::new();
                hasher.update(s.as_bytes());
                m.insert(
                    format!(":begins_with_{:x}", hasher.finalize()),
                    super::AttributeValue::S(s),
                );
            }
            Self::Contains(_path, s) => {
                let mut hasher = Md5::new();
                hasher.update(s.as_bytes());
                m.insert(
                    format!(":contains_{:x}", hasher.finalize()),
                    super::AttributeValue::S(s),
                );
            }
            Self::ContainsValue(_path, placeholder, value) => {
                m.insert(placeholder, *value);
            }
            _ => {}
        }
        m
    }
}

impl std::fmt::Display for ConditionComparisonExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eq(left, _, right, _) => write!(f, "{left} = {right}"),
            Self::Compare(left, _, operator, right, _) => write!(f, "{left} {operator} {right}"),
            Self::Between(path, lower, _, upper, _) => {
                write!(f, "{path} BETWEEN {lower} AND {upper}")
            }
            Self::In(path, values) => {
                write!(f, "{path} IN (")?;
                for (index, (placeholder, _)) in values.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{placeholder}")?;
                }
                write!(f, ")")
            }
            Self::Size(path, operator, placeholder, _) => {
                write!(f, "size({path}) {operator} {placeholder}")
            }
        }
    }
}

impl super::ToAttrNames for ConditionComparisonExpression {
    fn to_attr_names(&self) -> super::AttributeNames {
        let mut m: super::AttributeNames = std::collections::HashMap::new();
        match self {
            Self::Eq(left, _, right, _) | Self::Compare(left, _, _, right, _) => {
                if let AttrOrPlaceholder::Attr(l) = left {
                    m = super::merge_map(m, l.attribute_names());
                }
                if let AttrOrPlaceholder::Attr(r) = right {
                    m = super::merge_map(m, r.attribute_names());
                }
            }
            Self::Size(path, _, _, _) => {
                m = super::merge_map(m, path.attribute_names());
            }
            Self::Between(path, _, _, _, _) | Self::In(path, _) => {
                m = super::merge_map(m, path.attribute_names());
            }
        }
        m
    }
}

impl super::IntoAttrValues for ConditionComparisonExpression {
    fn into_attr_values(self) -> super::AttributeValues {
        let mut m: super::AttributeValues = std::collections::HashMap::new();

        match self {
            Self::Eq(left, left_value, right, right_value)
            | Self::Compare(left, left_value, _, right, right_value) => {
                if let Some(left_value) = left_value {
                    m.insert(left.to_string(), left_value);
                }
                if let Some(right_value) = right_value {
                    m.insert(right.to_string(), right_value);
                }
            }
            Self::Size(_, _, placeholder, value) => {
                m.insert(placeholder, value);
            }
            Self::Between(_, lower, lower_value, upper, upper_value) => {
                m.insert(lower, lower_value);
                m.insert(upper, upper_value);
            }
            Self::In(_, values) => {
                m.extend(values);
            }
        }
        m
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttrOrPlaceholder {
    Attr(super::AttrPath),
    Placeholder(String),
}

impl std::fmt::Display for AttrOrPlaceholder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Placeholder(p) => write!(f, ":{p}"),
            Self::Attr(a) => write!(f, "{a}"),
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

impl std::fmt::Display for Cond {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Func(func) => write!(f, "{func}"),
            Self::Cmp(cmp) => write!(f, "{cmp}"),
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

pub struct ConditionGroup<T> {
    expression: ConditionString,
    names: super::AttributeNames,
    values: super::AttributeValues,
    _token: std::marker::PhantomData<fn() -> T>,
}

impl<T> ConditionBuilder<T> for ConditionGroup<T> {
    fn build(
        self,
    ) -> (
        ConditionString,
        super::AttributeNames,
        super::AttributeValues,
    ) {
        (self.expression, self.names, self.values)
    }
}

pub trait ConditionBuilder<T>: Sized {
    fn build(
        self,
    ) -> (
        ConditionString,
        super::AttributeNames,
        super::AttributeValues,
    );

    /// Negates this entire condition, including any AND or OR operands.
    fn not(self) -> ConditionGroup<T> {
        let (expression, names, values) = self.build();
        ConditionGroup {
            expression: format!("NOT ({expression})"),
            names,
            values,
            _token: std::marker::PhantomData,
        }
    }

    /// Combines complete conditions and groups both operands explicitly.
    fn and(self, right: impl ConditionBuilder<T>) -> ConditionGroup<T> {
        let (left, left_names, left_values) = self.build();
        let (right, right_names, right_values) = right.build();
        ConditionGroup {
            expression: format!("({left}) AND ({right})"),
            names: super::merge_map(left_names, right_names),
            values: super::merge_map(left_values, right_values),
            _token: std::marker::PhantomData,
        }
    }

    /// Combines complete conditions and groups both operands explicitly.
    fn or(self, right: impl ConditionBuilder<T>) -> ConditionGroup<T> {
        let (left, left_names, left_values) = self.build();
        let (right, right_names, right_values) = right.build();
        ConditionGroup {
            expression: format!("({left}) OR ({right})"),
            names: super::merge_map(left_names, right_names),
            values: super::merge_map(left_values, right_values),
            _token: std::marker::PhantomData,
        }
    }
}
