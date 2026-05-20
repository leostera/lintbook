use serde::{Deserialize, Serialize};

use crate::Term;
use crate::error::{Error, Result};

/// A predicate applied to a list of terms.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Atom {
    pub predicate: String,
    pub args: Vec<Term>,
}

impl Atom {
    pub fn new(predicate: impl Into<String>, args: Vec<Term>) -> Result<Self> {
        let predicate = predicate.into();
        if predicate.is_empty() {
            return Err(Error::EmptyPredicate);
        }
        Ok(Self { predicate, args })
    }

    pub fn is_ground(&self) -> bool {
        self.args.iter().all(Term::is_const)
    }

    pub fn variables(&self) -> Vec<&str> {
        self.args
            .iter()
            .flat_map(Term::variables)
            .collect::<Vec<_>>()
    }
}

impl std::fmt::Display for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let args = self
            .args
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "{}({args})", self.predicate)
    }
}

/// One clause in a query body.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Clause {
    Atom(Atom),
    Negated(Atom),
    Builtin { name: String, args: Vec<Term> },
}

impl Clause {
    pub fn atom(atom: Atom) -> Self {
        Self::Atom(atom)
    }

    pub fn negated(atom: Atom) -> Self {
        Self::Negated(atom)
    }

    pub fn builtin(name: impl Into<String>, args: Vec<Term>) -> Self {
        Self::Builtin {
            name: name.into(),
            args,
        }
    }

    pub fn variables(&self) -> Vec<&str> {
        match self {
            Self::Atom(atom) | Self::Negated(atom) => atom.variables(),
            Self::Builtin { args, .. } => args.iter().flat_map(Term::variables).collect(),
        }
    }
}

/// A read-only Datalog query.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Query {
    Single(Atom),
    Multi(Vec<Clause>),
}

impl Query {
    pub fn single(atom: Atom) -> Self {
        Self::Single(atom)
    }

    pub fn multi(clauses: Vec<Clause>) -> Result<Self> {
        if clauses.is_empty() {
            return Err(Error::EmptyQuery);
        }
        Ok(Self::Multi(clauses))
    }

    pub fn clauses(&self) -> Vec<Clause> {
        match self {
            Self::Single(atom) => vec![Clause::Atom(atom.clone())],
            Self::Multi(clauses) => clauses.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Atom, Clause, Query, Result, Term, Value};

    #[test]
    fn atom_requires_a_non_empty_predicate() {
        assert!(Atom::new("", vec![]).is_err());
    }

    #[test]
    fn atom_formats_like_datalog() -> Result<()> {
        let atom = Atom::new(
            "spotify:displayName",
            vec![
                Term::variable("Album")?,
                Term::constant(Value::string("2112")),
            ],
        )?;

        assert_eq!(atom.to_string(), "spotify:displayName(Album, \"2112\")");
        Ok(())
    }

    #[test]
    fn atom_reports_when_it_is_ground() -> Result<()> {
        let ground = Atom::new(
            "edge",
            vec![
                Term::constant(Value::integer(1)),
                Term::constant(Value::integer(2)),
            ],
        )?;
        let non_ground = Atom::new("edge", vec![Term::variable("X")?, Term::wildcard()])?;

        assert!(ground.is_ground());
        assert!(!non_ground.is_ground());
        Ok(())
    }

    #[test]
    fn query_multi_requires_at_least_one_clause() {
        assert!(Query::multi(vec![]).is_err());
    }

    #[test]
    fn query_clauses_expose_single_and_multi_shapes() -> Result<()> {
        let atom = Atom::new("person", vec![Term::variable("X")?])?;
        let single = Query::single(atom.clone());
        let multi = Query::multi(vec![Clause::atom(atom.clone())])?;

        assert_eq!(single.clauses(), vec![Clause::atom(atom.clone())]);
        assert_eq!(multi.clauses(), vec![Clause::atom(atom)]);
        Ok(())
    }

    #[test]
    fn clause_collects_variables() -> Result<()> {
        let clause = Clause::builtin(
            "=",
            vec![Term::variable("X")?, Term::constant(Value::integer(42))],
        );

        assert_eq!(clause.variables(), vec!["X"]);
        Ok(())
    }
}
