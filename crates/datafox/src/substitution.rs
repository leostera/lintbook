use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{Atom, Term, Value};

/// A deterministic mapping from variable names to concrete values.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Substitution {
    bindings: BTreeMap<String, Value>,
}

impl Substitution {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn singleton(var: impl Into<String>, value: Value) -> Self {
        let mut substitution = Self::new();
        substitution = substitution.bind(var, value);
        substitution
    }

    pub fn from_bindings(bindings: impl IntoIterator<Item = (String, Value)>) -> Self {
        Self {
            bindings: bindings.into_iter().collect(),
        }
    }

    pub fn bind(mut self, var: impl Into<String>, value: Value) -> Self {
        self.bindings.insert(var.into(), value);
        self
    }

    pub fn lookup(&self, var: &str) -> Option<&Value> {
        self.bindings.get(var)
    }

    pub fn contains(&self, var: &str) -> bool {
        self.bindings.contains_key(var)
    }

    pub fn unbind(mut self, var: &str) -> Self {
        self.bindings.remove(var);
        self
    }

    pub fn merge(&self, other: &Self) -> Option<Self> {
        let mut merged = self.clone();

        for (var, value) in &other.bindings {
            match merged.lookup(var) {
                Some(existing) if existing != value => return None,
                Some(_) => {}
                None => {
                    merged.bindings.insert(var.clone(), value.clone());
                }
            }
        }

        Some(merged)
    }

    pub fn extend(&self, bindings: impl IntoIterator<Item = (String, Value)>) -> Option<Self> {
        self.merge(&Self::from_bindings(bindings))
    }

    pub fn apply_to_term(&self, term: &Term) -> Term {
        match term {
            Term::Var(var) => self
                .lookup(var)
                .cloned()
                .map(Term::constant)
                .unwrap_or_else(|| term.clone()),
            _ => term.clone(),
        }
    }

    pub fn apply_to_atom(&self, atom: &Atom) -> Atom {
        Atom {
            predicate: atom.predicate.clone(),
            args: atom
                .args
                .iter()
                .map(|term| self.apply_to_term(term))
                .collect(),
        }
    }

    pub fn apply_to_terms(&self, terms: &[Term]) -> Option<Vec<Value>> {
        terms
            .iter()
            .map(|term| match self.apply_to_term(term) {
                Term::Const(value) => Some(value),
                Term::Var(_) | Term::Wildcard => None,
            })
            .collect()
    }

    pub fn bindings(&self) -> impl Iterator<Item = (&str, &Value)> {
        self.bindings
            .iter()
            .map(|(var, value)| (var.as_str(), value))
    }

    pub fn variables(&self) -> impl Iterator<Item = &str> {
        self.bindings.keys().map(String::as_str)
    }

    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    pub fn len(&self) -> usize {
        self.bindings.len()
    }
}

impl std::fmt::Display for Substitution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            return write!(f, "{{}}");
        }

        let bindings = self
            .bindings
            .iter()
            .map(|(var, value)| format!("{var}→{value}"))
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "{{{bindings}}}")
    }
}

#[cfg(test)]
mod tests {
    use crate::{Atom, Result, Substitution, Term, Value};

    #[test]
    fn merge_combines_compatible_bindings() {
        let left = Substitution::new().bind("X", Value::integer(1));
        let right = Substitution::new().bind("Y", Value::integer(2));

        let merged = left.merge(&right).expect("compatible merge");

        assert_eq!(merged.lookup("X"), Some(&Value::integer(1)));
        assert_eq!(merged.lookup("Y"), Some(&Value::integer(2)));
    }

    #[test]
    fn merge_rejects_conflicting_bindings() {
        let left = Substitution::new().bind("X", Value::integer(1));
        let right = Substitution::new().bind("X", Value::integer(2));

        assert!(left.merge(&right).is_none());
    }

    #[test]
    fn apply_to_term_replaces_bound_variables() -> Result<()> {
        let substitution = Substitution::new().bind("Album", Value::string("spotify:album:2112"));
        let applied = substitution.apply_to_term(&Term::variable("Album")?);

        assert_eq!(applied, Term::constant(Value::string("spotify:album:2112")));
        Ok(())
    }

    #[test]
    fn apply_to_atom_rewrites_all_bound_terms() -> Result<()> {
        let substitution = Substitution::new()
            .bind("Album", Value::string("spotify:album:2112"))
            .bind("Name", Value::string("2112"));
        let atom = Atom::new(
            "spotify:displayName",
            vec![Term::variable("Album")?, Term::variable("Name")?],
        )?;

        let applied = substitution.apply_to_atom(&atom);

        assert_eq!(
            applied,
            Atom::new(
                "spotify:displayName",
                vec![
                    Term::constant(Value::string("spotify:album:2112")),
                    Term::constant(Value::string("2112")),
                ],
            )?
        );
        Ok(())
    }

    #[test]
    fn apply_to_terms_requires_all_terms_to_be_ground() -> Result<()> {
        let substitution = Substitution::new().bind("X", Value::integer(1));

        assert_eq!(
            substitution
                .apply_to_terms(&[Term::variable("X")?, Term::constant(Value::integer(2)),]),
            Some(vec![Value::integer(1), Value::integer(2)])
        );
        assert_eq!(substitution.apply_to_terms(&[Term::variable("Y")?]), None);
        Ok(())
    }
}
