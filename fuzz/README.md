# datafox fuzzing

These targets exercise the `datafox` parser and in-memory evaluator with libFuzzer.

Install the runner once:

```bash
cargo install cargo-fuzz
rustup toolchain install nightly
```

Run bounded smoke jobs:

```bash
cargo +nightly fuzz run parse_queries -- -runs=10000
cargo +nightly fuzz run evaluate_in_memory -- -runs=10000
```

For longer local runs, prefer a time limit:

```bash
cargo +nightly fuzz run parse_queries -- -max_total_time=300
cargo +nightly fuzz run evaluate_in_memory -- -max_total_time=300
```

Cargo-fuzz expands `fuzz/corpus/<target>` while it discovers new coverage and writes crashes under `fuzz/artifacts/`. Curated seed files are checked in. Generated corpus additions should be reviewed or discarded, and crash artifacts are ignored.
