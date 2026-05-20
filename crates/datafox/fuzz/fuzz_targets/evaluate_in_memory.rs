#![no_main]

use datafox::{Evaluator, InMemoryStorage, Value};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() > 512 {
        return;
    }

    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };
    let Ok(queries) = datafox::parse_queries(source) else {
        return;
    };

    let storage = InMemoryStorage::from_facts([
        (
            "edge".to_string(),
            vec![
                vec![Value::integer(1), Value::integer(2)],
                vec![Value::integer(2), Value::integer(3)],
                vec![Value::integer(3), Value::integer(3)],
            ],
        ),
        (
            "displayName".to_string(),
            vec![
                vec![Value::string("rush"), Value::string("Rush")],
                vec![Value::string("yes"), Value::string("Yes")],
            ],
        ),
        (
            "text".to_string(),
            vec![
                vec![Value::string("node-1"), Value::string("dbg!")],
                vec![Value::string("node-2"), Value::string("println!")],
            ],
        ),
    ]);

    for query in queries.into_iter().take(16) {
        let _ = Evaluator::evaluate_in_memory(&storage, &query);
    }
});
