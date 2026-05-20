use serde::{Deserialize, Serialize};

/// Concrete constant values that can appear in Datalog facts and queries.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Value {
    Integer(i64),
    String(String),
}

impl Value {
    pub fn integer(value: i64) -> Self {
        Self::Integer(value)
    }

    pub fn string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Self::Integer(i64::from(value))
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Integer(value) => write!(f, "{value}"),
            Self::String(value) => write!(f, "\"{value}\""),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Value;

    #[test]
    fn value_formats_like_datalog_literals() {
        assert_eq!(Value::integer(42).to_string(), "42");
        assert_eq!(Value::string("rush").to_string(), "\"rush\"");
    }

    #[test]
    fn value_from_common_scalars_is_convenient() {
        assert_eq!(Value::from(42_i64), Value::integer(42));
        assert_eq!(Value::from(42_i32), Value::integer(42));
        assert_eq!(Value::from("2112"), Value::string("2112"));
        assert_eq!(Value::from("2112".to_string()), Value::string("2112"));
    }
}
