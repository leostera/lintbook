use std::collections::BTreeMap;

use async_trait::async_trait;
use tokio::sync::mpsc;
use tracing::debug;

use crate::{Result, Value};

pub type FactTuple = Vec<Value>;
pub type TupleStream = mpsc::Receiver<Result<FactTuple>>;

const DEFAULT_STREAM_BUFFER: usize = 64;

/// Snapshot-oriented read-only storage interface for Datalog queries.
#[async_trait]
pub trait Storage {
    async fn get_facts_matching(
        &self,
        predicate: &str,
        pattern: Vec<Option<Value>>,
    ) -> Result<TupleStream>;
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InMemoryStorage {
    facts: BTreeMap<String, Vec<FactTuple>>,
    indexes: BTreeMap<String, BTreeMap<(usize, Value), Vec<usize>>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_facts(facts: impl IntoIterator<Item = (String, Vec<FactTuple>)>) -> Self {
        let mut storage = Self::new();
        for (predicate, tuples) in facts {
            for tuple in tuples {
                storage.insert(predicate.clone(), tuple);
            }
        }
        storage
    }

    pub fn insert(&mut self, predicate: impl Into<String>, tuple: FactTuple) {
        let predicate = predicate.into();
        let tuple_index = self.facts.get(&predicate).map_or(0, Vec::len);

        for (value_index, value) in tuple.iter().cloned().enumerate() {
            self.indexes
                .entry(predicate.clone())
                .or_default()
                .entry((value_index, value))
                .or_default()
                .push(tuple_index);
        }

        self.facts.entry(predicate).or_default().push(tuple);
    }

    pub fn facts_matching<'a>(
        &'a self,
        predicate: &str,
        pattern: &[Option<Value>],
    ) -> Vec<&'a FactTuple> {
        let Some(facts) = self.facts.get(predicate) else {
            return Vec::new();
        };

        let best_index = pattern
            .iter()
            .enumerate()
            .filter_map(|(value_index, value)| {
                let value = value.as_ref()?;
                let tuple_indexes = self
                    .indexes
                    .get(predicate)?
                    .get(&(value_index, value.clone()))?;
                Some(tuple_indexes)
            })
            .min_by_key(|tuple_indexes| tuple_indexes.len());

        if let Some(tuple_indexes) = best_index {
            return tuple_indexes
                .iter()
                .filter_map(|tuple_index| facts.get(*tuple_index))
                .filter(|tuple| matches_pattern(pattern, tuple))
                .collect();
        }

        facts
            .iter()
            .filter(|tuple| matches_pattern(pattern, tuple))
            .collect()
    }
}

#[async_trait]
impl Storage for InMemoryStorage {
    async fn get_facts_matching(
        &self,
        predicate: &str,
        pattern: Vec<Option<Value>>,
    ) -> Result<TupleStream> {
        let tuples = self
            .facts_matching(predicate, &pattern)
            .into_iter()
            .cloned()
            .map(Ok)
            .collect::<Vec<_>>();
        debug!(match_count = tuples.len(), "filtered in-memory tuples");

        let (tx, rx) = mpsc::channel(tuples.len().max(DEFAULT_STREAM_BUFFER));
        tokio::spawn(async move {
            for tuple in tuples {
                if tx.send(tuple).await.is_err() {
                    break;
                }
            }
        });

        Ok(rx)
    }
}

pub fn matches_pattern(pattern: &[Option<Value>], tuple: &[Value]) -> bool {
    pattern.len() == tuple.len()
        && pattern
            .iter()
            .zip(tuple)
            .all(|(pattern, value)| match pattern {
                Some(pattern) => pattern == value,
                None => true,
            })
}

#[cfg(test)]
mod tests {
    use tokio::runtime::Runtime;

    use crate::{InMemoryStorage, Storage, Value, matches_pattern};

    #[test]
    fn matches_pattern_treats_none_as_wildcard() {
        assert!(matches_pattern(
            &[Some(Value::integer(1)), None],
            &[Value::integer(1), Value::integer(2)],
        ));
        assert!(!matches_pattern(
            &[Some(Value::integer(1)), None],
            &[Value::integer(2), Value::integer(3)],
        ));
    }

    #[test]
    fn in_memory_storage_filters_matching_tuples() {
        let storage = InMemoryStorage::from_facts([(
            "edge".to_string(),
            vec![
                vec![Value::integer(1), Value::integer(2)],
                vec![Value::integer(2), Value::integer(3)],
            ],
        )]);

        let runtime = Runtime::new().expect("runtime");
        let tuples = runtime.block_on(async {
            let mut tuples = storage
                .get_facts_matching("edge", vec![Some(Value::integer(1)), None])
                .await
                .expect("tuples");
            let mut results = Vec::new();
            while let Some(tuple) = tuples.recv().await {
                results.push(tuple.expect("tuple result"));
            }
            results
        });

        assert_eq!(tuples, vec![vec![Value::integer(1), Value::integer(2)]]);
    }
}
