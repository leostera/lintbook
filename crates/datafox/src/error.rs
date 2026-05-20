use thiserror::Error;

use crate::Diagnostic;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("variable names must not be empty")]
    EmptyVariableName,
    #[error("atom predicate names must not be empty")]
    EmptyPredicate,
    #[error("queries must contain at least one clause")]
    EmptyQuery,
    #[error("single-goal queries require exactly one positive atom")]
    InvalidSingleQueryShape,
    #[error("multi-goal queries are not implemented yet")]
    UnsupportedMultiQuery,
    #[error("negated clauses are not implemented yet")]
    UnsupportedNegation,
    #[error("unsupported builtin clause `{name}`")]
    UnsupportedBuiltin { name: String },
    #[error("builtin `{name}` expected {expected} arguments, found {found}")]
    BuiltinArityMismatch {
        name: String,
        expected: usize,
        found: usize,
    },
    #[error("builtin `{name}` expected {expected}")]
    BuiltinTypeMismatch { name: String, expected: String },
    #[error("builtin `{name}` requires ground arguments before it can be evaluated")]
    UngroundedBuiltin { name: String },
    #[error("arity mismatch for predicate `{predicate}`: expected {expected}, found {found}")]
    ArityMismatch {
        predicate: String,
        expected: usize,
        found: usize,
    },
    #[error("query parse failed")]
    Parse { diagnostics: Vec<Diagnostic> },
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use crate::{Diagnostic, Error, Span};

    #[test]
    fn parse_errors_preserve_diagnostics() {
        let error = Error::Parse {
            diagnostics: vec![
                Diagnostic::new("expected `)`")
                    .with_span(Span::new(4, 5))
                    .with_found(","),
            ],
        };

        match error {
            Error::Parse { diagnostics } => {
                assert_eq!(diagnostics.len(), 1);
                assert_eq!(diagnostics[0].message, "expected `)`");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
