# datafox

`datafox` is a small Datalog parser and streaming query engine for querying caller-owned facts.

It was built for lintbook rule evaluation, but the crate is standalone: provide facts through a `Storage`, parse read-only queries, and evaluate substitutions.

```toml
[dependencies]
datafox = "0.1"
```

```rust
use datafox::{Evaluator, InMemoryStorage, Value, parse_query};

fn main() -> datafox::Result<()> {
    let storage = InMemoryStorage::from_facts([(
        "edge".to_string(),
        vec![
            vec![Value::integer(1), Value::integer(2)],
            vec![Value::integer(2), Value::integer(3)],
        ],
    )]);

    let query = parse_query("edge(From, 2)")?;
    let results = Evaluator::evaluate_in_memory(&storage, &query)?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].lookup("From"), Some(&Value::integer(1)));
    Ok(())
}
```

## Query Syntax

| Form | Example |
| --- | --- |
| Atom | `edge(From, To)` |
| Variable | `Name` |
| String constant | `"dbg!"` |
| Integer constant | `42` |
| Wildcard | `_` |
| Conjunction | `node(Node), text(Node, Text)` |
| Negation | `node(Node), !test(Node)` |
| Query set | `node(Node); edge(From, To)` |
| Quoted predicate | `'local://schema/name'(Entity, Value)` |

Builtins are available as clauses:

| Builtin | Example |
| --- | --- |
| Equality and order | `Start < End`, `A = B` |
| String matching | `startsWith(Name, "lint")` |
| Negative string matching | `notContains(Text, "dbg!")` |
| Regex matching | `matchesRegex(Text, "^dbg!")` |
| Temporal aliases | `before(Start, End)`, `after(End, Start)` |

Negated atoms and builtin arguments must be grounded by earlier clauses. Evaluation is read-only and snapshot-oriented; facts are supplied by the caller.
