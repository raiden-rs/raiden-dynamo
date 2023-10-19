use std::fmt;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NextToken(String);

impl NextToken {
    pub fn new(token: impl Into<String>) -> Self {
        Self(token.into())
    }
    pub fn into_attr_values(self) -> Result<super::AttributeValues, super::RaidenError> {
        let decoded = match base64::decode(self.0) {
            Ok(decoded) => decoded,
            Err(_) => return Err(super::RaidenError::NextTokenDecodeError),
        };
        let s = match std::str::from_utf8(&decoded[..]) {
            Ok(s) => s,
            Err(_) => return Err(super::RaidenError::NextTokenDecodeError),
        };

        let deserialized: std::collections::HashMap<String, super::AttributeValue> =
            match serde_json::from_str(s) {
                Ok(deserialized) => deserialized,
                Err(_) => return Err(super::RaidenError::NextTokenDecodeError),
            };
        Ok(deserialized)
    }

    pub fn from_attr(key: &super::AttributeValues) -> Self {
        let serialized = serde_json::to_string(key).expect("should serialize");
        Self(base64::encode(serialized))
    }
}

impl fmt::Display for NextToken {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
