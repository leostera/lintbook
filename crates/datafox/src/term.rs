use serde::{Deserialize, Serialize};

use crate::Value;
use crate::error::{Error, Result};

/// One Datalog term.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Term {
    Var(String),
    Const(Value),
    Wildcard,
}

impl Term {
    pub fn variable(name: impl Into<String>) -> Result<Self> {
        let name = name.into();
        if name.is_empty() {
            return Err(Error::EmptyVariableName);
        }
        Ok(Self::Var(name))
    }

    pub fn constant(value: Value) -> Self {
        Self::Const(value)
    }

    pub fn wildcard() -> Self {
        Self::Wildcard
    }

    pub fn is_var(&self) -> bool {
        matches!(self, Self::Var(_))
    }

    pub fn is_const(&self) -> bool {
        matches!(self, Self::Const(_))
    }

    pub fn is_wildcard(&self) -> bool {
        matches!(self, Self::Wildcard)
    }

    pub fn var_name(&self) -> Option<&str> {
        match self {
            Self::Var(name) => Some(name),
            _ => None,
        }
    }

    pub fn const_value(&self) -> Option<&Value> {
        match self {
            Self::Const(value) => Some(value),
            _ => None,
        }
    }

    pub fn variables(&self) -> Vec<&str> {
        self.var_name().into_iter().collect()
    }
}

impl std::fmt::Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Var(name) => write!(f, "{name}"),
            Self::Const(value) => write!(f, "{value}"),
            Self::Wildcard => write!(f, "_"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Result, Term, Value};

    #[test]
    fn term_variable_requires_a_non_empty_name() {
        assert!(Term::variable("").is_err());
    }

    #[test]
    fn term_reports_its_kind() -> Result<()> {
        let var = Term::variable("Album")?;
        let constant = Term::constant(Value::string("2112"));
        let wildcard = Term::wildcard();

        assert!(var.is_var());
        assert!(constant.is_const());
        assert!(wildcard.is_wildcard());
        Ok(())
    }
}
