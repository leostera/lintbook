#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() > 4096 {
        return;
    }

    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };

    let _ = datafox::parse_query(source);
    let _ = datafox::parse_queries(source);
});
