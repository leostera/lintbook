use std::time::Instant;

use datafox::{Evaluator, InMemoryStorage, Result, Value, parse_query};

fn main() -> Result<()> {
    let mut edges = Vec::with_capacity(20_000);
    let mut labels = Vec::with_capacity(20_001);

    for node in 0..20_000 {
        edges.push(vec![Value::integer(node), Value::integer(node + 1)]);
        labels.push(vec![
            Value::integer(node),
            Value::string(format!("node-{node}")),
        ]);
    }
    labels.push(vec![Value::integer(20_000), Value::string("node-20000")]);

    let storage =
        InMemoryStorage::from_facts([("edge".to_string(), edges), ("label".to_string(), labels)]);
    let query = parse_query(r#"edge(From, To), label(To, Name), contains(Name, "999")"#)?;

    let iterations = std::env::var("DATAFOX_PROFILE_ITERS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(200);

    let start = Instant::now();
    let mut total = 0usize;
    for _ in 0..iterations {
        total += Evaluator::evaluate_in_memory(&storage, &query)?.len();
    }

    let elapsed = start.elapsed();
    println!(
        "iterations={iterations} total_matches={total} elapsed_ms={:.3}",
        elapsed.as_secs_f64() * 1_000.0
    );

    Ok(())
}
