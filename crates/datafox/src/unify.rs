use crate::{Atom, Error, Result, Substitution, Term, Value};

/// Stateless helpers for unification and atom-to-tuple matching.
pub struct Unifier;

impl Unifier {
    pub fn unify_terms(
        substitution: &Substitution,
        left: &Term,
        right: &Term,
    ) -> Option<Substitution> {
        let left = substitution.apply_to_term(left);
        let right = substitution.apply_to_term(right);

        match (left, right) {
            (Term::Const(left), Term::Const(right)) => {
                (left == right).then(|| substitution.clone())
            }
            (Term::Var(var), Term::Const(value)) | (Term::Const(value), Term::Var(var)) => {
                Some(substitution.clone().bind(var, value))
            }
            (Term::Var(left), Term::Var(right)) => {
                if left == right {
                    Some(substitution.clone())
                } else {
                    Some(substitution.clone())
                }
            }
            (Term::Wildcard, _) | (_, Term::Wildcard) => Some(substitution.clone()),
        }
    }

    pub fn unify_term_lists(
        substitution: &Substitution,
        left: &[Term],
        right: &[Term],
    ) -> Option<Substitution> {
        if left.len() != right.len() {
            return None;
        }

        left.iter()
            .zip(right)
            .try_fold(substitution.clone(), |substitution, (left, right)| {
                Self::unify_terms(&substitution, left, right)
            })
    }

    pub fn unify_atoms(
        substitution: &Substitution,
        left: &Atom,
        right: &Atom,
    ) -> Option<Substitution> {
        if left.predicate != right.predicate {
            return None;
        }

        Self::unify_term_lists(substitution, &left.args, &right.args)
    }

    pub fn match_atom(
        substitution: &Substitution,
        atom: &Atom,
        tuple: &[Value],
    ) -> Result<Option<Substitution>> {
        if atom.args.len() != tuple.len() {
            return Err(Error::ArityMismatch {
                predicate: atom.predicate.clone(),
                expected: atom.args.len(),
                found: tuple.len(),
            });
        }

        let mut current = substitution.clone();
        for (term, value) in atom.args.iter().zip(tuple) {
            match term {
                Term::Const(expected) if expected != value => return Ok(None),
                Term::Const(_) | Term::Wildcard => {}
                Term::Var(variable) => match current.lookup(variable) {
                    Some(existing) if existing != value => return Ok(None),
                    Some(_) => {}
                    None => current = current.bind(variable.clone(), value.clone()),
                },
            }
        }

        Ok(Some(current))
    }

    pub fn ground_term(substitution: &Substitution, term: &Term) -> Option<Value> {
        match term {
            Term::Const(value) => Some(value.clone()),
            Term::Var(variable) => substitution.lookup(variable).cloned(),
            Term::Wildcard => None,
        }
    }

    pub fn ground_terms(substitution: &Substitution, terms: &[Term]) -> Option<Vec<Value>> {
        terms
            .iter()
            .map(|term| Self::ground_term(substitution, term))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Atom, Error, Result, Substitution, Term, Unifier, Value};

    #[test]
    fn unify_terms_binds_variables_to_constants() -> Result<()> {
        let substitution = Substitution::new();
        let unified = Unifier::unify_terms(
            &substitution,
            &Term::variable("X")?,
            &Term::constant(Value::integer(42)),
        )
        .expect("unified");

        assert_eq!(unified.lookup("X"), Some(&Value::integer(42)));
        Ok(())
    }

    #[test]
    fn unify_terms_rejects_distinct_constants() {
        let substitution = Substitution::new();
        let unified = Unifier::unify_terms(
            &substitution,
            &Term::constant(Value::integer(1)),
            &Term::constant(Value::integer(2)),
        );

        assert!(unified.is_none());
    }

    #[test]
    fn unify_atoms_respects_predicate_and_argument_shapes() -> Result<()> {
        let left = Atom::new(
            "edge",
            vec![Term::variable("X")?, Term::constant(Value::integer(2))],
        )?;
        let right = Atom::new(
            "edge",
            vec![Term::constant(Value::integer(1)), Term::variable("Y")?],
        )?;

        let unified = Unifier::unify_atoms(&Substitution::new(), &left, &right).expect("unified");

        assert_eq!(unified.lookup("X"), Some(&Value::integer(1)));
        assert_eq!(unified.lookup("Y"), Some(&Value::integer(2)));
        Ok(())
    }

    #[test]
    fn match_atom_returns_bindings_for_matching_tuples() -> Result<()> {
        let atom = Atom::new(
            "spotify:displayName",
            vec![
                Term::variable("Album")?,
                Term::constant(Value::string("2112")),
            ],
        )?;

        let matched = Unifier::match_atom(
            &Substitution::new(),
            &atom,
            &[Value::string("spotify:album:2112"), Value::string("2112")],
        )?
        .expect("matched");

        assert_eq!(
            matched.lookup("Album"),
            Some(&Value::string("spotify:album:2112"))
        );
        Ok(())
    }

    #[test]
    fn match_atom_returns_none_for_non_matching_tuples() -> Result<()> {
        let atom = Atom::new(
            "spotify:displayName",
            vec![
                Term::variable("Album")?,
                Term::constant(Value::string("2112")),
            ],
        )?;

        let matched = Unifier::match_atom(
            &Substitution::new(),
            &atom,
            &[
                Value::string("spotify:album:signals"),
                Value::string("Signals"),
            ],
        )?;

        assert!(matched.is_none());
        Ok(())
    }

    #[test]
    fn match_atom_reports_arity_mismatches() -> Result<()> {
        let atom = Atom::new("edge", vec![Term::variable("X")?, Term::variable("Y")?])?;
        let error = Unifier::match_atom(&Substitution::new(), &atom, &[Value::integer(1)])
            .expect_err("arity mismatch");

        assert_eq!(
            error,
            Error::ArityMismatch {
                predicate: "edge".to_string(),
                expected: 2,
                found: 1,
            }
        );
        Ok(())
    }

    #[test]
    fn ground_terms_requires_fully_bound_inputs() -> Result<()> {
        let substitution = Substitution::new()
            .bind("X", Value::integer(1))
            .bind("Y", Value::integer(2));

        assert_eq!(
            Unifier::ground_terms(&substitution, &[Term::variable("X")?, Term::variable("Y")?]),
            Some(vec![Value::integer(1), Value::integer(2)])
        );
        assert_eq!(
            Unifier::ground_terms(&substitution, &[Term::variable("X")?, Term::variable("Z")?]),
            None
        );
        Ok(())
    }
}
