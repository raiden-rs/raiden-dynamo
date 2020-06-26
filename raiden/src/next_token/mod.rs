#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NextToken(String);

impl NextToken {
    pub fn new(token: impl Into<String>) -> Self {
        Self(token.into())
    }
    pub fn into_attr_values(self) -> Result<super::AttributeValues, super::RaidenError> {
        let decoded =
            &base64::decode(&self.0).unwrap_or(Err(super::RaidenError::NextTokenDecodeError)?)[..];
        let s =
            &std::str::from_utf8(decoded).unwrap_or(Err(super::RaidenError::NextTokenDecodeError)?);
        let deserialized: std::collections::HashMap<String, super::AttributeValue> =
            serde_json::from_str(s).unwrap_or(Err(super::RaidenError::NextTokenDecodeError)?);
        Ok(deserialized)
    }

    pub fn from_attr(key: &super::AttributeValues) -> Self {
        let serialized = serde_json::to_string(key).expect("should serialize");
        Self(base64::encode(&serialized))
    }
}
