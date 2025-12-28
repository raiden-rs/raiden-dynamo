use std::fmt;

use base64::{engine::general_purpose::STANDARD, Engine};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NextToken(String);

impl NextToken {
    pub fn new(token: impl Into<String>) -> Self {
        Self(token.into())
    }
    #[allow(clippy::result_large_err)]
    pub fn into_attr_values(self) -> Result<super::AttributeValues, super::RaidenError> {
        let decoded = match STANDARD.decode(self.0) {
            Ok(decoded) => decoded,
            Err(_) => return Err(super::RaidenError::NextTokenDecodeError),
        };
        let s = match std::str::from_utf8(&decoded[..]) {
            Ok(s) => s,
            Err(_) => return Err(super::RaidenError::NextTokenDecodeError),
        };

        crate::deserialize_attr_value(s)
    }

    pub fn from_attr(key: &super::AttributeValues) -> Self {
        let serialized = crate::serialize_attr_values(key);
        Self(STANDARD.encode(serialized))
    }
}

impl fmt::Display for NextToken {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
