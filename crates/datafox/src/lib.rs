#![forbid(unsafe_code)]
//! Datafox is a standalone Datalog parser and streaming query engine for facts.
//!
//! The crate is intentionally small: callers provide facts through [`Storage`],
//! parse read-only queries with [`parse_query`] or [`parse_queries`], and evaluate
//! them with [`Evaluator`].
//!
//! ```
//! use datafox::{Evaluator, InMemoryStorage, Value, parse_query};
//!
//! let storage = InMemoryStorage::from_facts([(
//!     "edge".to_string(),
//!     vec![
//!         vec![Value::integer(1), Value::integer(2)],
//!         vec![Value::integer(2), Value::integer(3)],
//!     ],
//! )]);
//! let query = parse_query("edge(From, 2)")?;
//! let results = Evaluator::evaluate_in_memory(&storage, &query)?;
//!
//! assert_eq!(results.len(), 1);
//! assert_eq!(results[0].lookup("From"), Some(&Value::integer(1)));
//! # Ok::<(), datafox::Error>(())
//! ```
//!
//! Public API:
//! - [`Value`] for Datalog constants.
//! - [`Term`] for variables, constants, and wildcards.
//! - [`Atom`], [`Clause`], and [`Query`] for query syntax trees.
//! - [`Diagnostic`] and [`parse_query`] for query parsing with context.
//! - [`Substitution`] and [`Unifier`] for binding and matching query variables.
//! - [`Storage`], [`Universe`], and [`Evaluator`] for snapshot-based query execution.
//! - [`Error`] and [`Result`] for typed failures.
//! - [`atom!`], [`var!`], [`lit!`], and [`subst!`] for test and call-site ergonomics.

mod ast;
mod diagnostic;
pub mod error;
mod evaluator;
mod parser;
mod storage;
mod substitution;
mod term;
mod unify;
mod universe;
mod value;

pub use ast::{Atom, Clause, Query};
pub use diagnostic::{Diagnostic, Span};
pub use error::{Error, Result};
pub use evaluator::{Evaluator, SubstitutionStream};
pub use parser::{parse_queries, parse_query};
pub use storage::{FactTuple, InMemoryStorage, Storage, TupleStream, matches_pattern};
pub use substitution::Substitution;
pub use term::Term;
pub use unify::Unifier;
pub use universe::Universe;
pub use value::Value;

#[macro_export]
macro_rules! atom {
    ($name:expr, $args:expr) => {{ $crate::Atom::new($name, $args).expect("invalid atom") }};
}

#[macro_export]
macro_rules! var {
    ($name:expr) => {{ $crate::Term::variable($name).expect("invalid variable") }};
}

#[macro_export]
macro_rules! lit {
    ($value:expr) => {{ $crate::Term::constant($value) }};
}

#[macro_export]
macro_rules! subst {
    ($(($name:expr, $value:expr)),* $(,)?) => {{
        $crate::Substitution::from_bindings(vec![
            $(($name.to_string(), $value)),*
        ])
    }};
}

#[cfg(test)]
mod tests {
    use crate::{Atom, Substitution, Term, Value};

    #[test]
    fn convenience_macros_build_terms_atoms_and_substitutions() {
        let atom = atom!(
            "spotify:displayName",
            vec![crate::var!("Album"), crate::lit!(Value::string("2112"))]
        );
        let substitution = crate::subst![
            ("Album", Value::string("spotify:album:2112")),
            ("Name", Value::string("2112")),
        ];

        assert_eq!(
            atom,
            Atom::new(
                "spotify:displayName",
                vec![
                    Term::variable("Album").expect("variable"),
                    Term::constant(Value::string("2112")),
                ],
            )
            .expect("atom"),
        );
        assert_eq!(
            substitution,
            Substitution::from_bindings(vec![
                ("Album".to_string(), Value::string("spotify:album:2112")),
                ("Name".to_string(), Value::string("2112")),
            ]),
        );
    }
}
