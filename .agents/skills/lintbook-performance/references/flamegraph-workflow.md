# Flamegraph Workflow

## Baselines

Keep cold-cache and warm-cache timings separate.

- Warm cache: existing `.lintbook/cache` is present and source file hashes are stable.
- Cold cache: `.lintbook/cache` is removed before the measured run.
- Build time: excluded by using `target/release/lintbook` for timing.
- Terminal output: can dominate small runs, so compare human, `--json`, and `--output json` separately when output behavior is relevant.

Use the same command, cache state, and repo revision before and after an optimization.

## Cargo Flamegraph Notes

On macOS, `cargo flamegraph` may use DTrace and may require root privileges. Do not use `--root` or sudo-like paths without user approval.

If `cargo flamegraph` records successfully but fails while collapsing xctrace XML, keep the timing result and fall back to direct xctrace export:

```bash
/Applications/Xcode.app/Contents/Developer/usr/bin/xctrace record \
  --template 'Time Profiler' \
  --output /tmp/lintbook-check-json.trace \
  --target-stdout /tmp/lintbook-xctrace.stdout \
  --launch -- target/release/lintbook check --json

/Applications/Xcode.app/Contents/Developer/usr/bin/xctrace export \
  --input /tmp/lintbook-check-json.trace \
  --xpath '/trace-toc/run[@number="1"]/data/table[@schema="time-profile"]' \
  > /tmp/lintbook-check-json-time.xml
```

The first profile often includes setup noise. If a change is small, take two profiles and compare the second run from each revision.

Prefer profiles that exercise the product path:

```bash
bash .agents/skills/lintbook-performance/scripts/flamegraph-check.sh -- --json
bash .agents/skills/lintbook-performance/scripts/flamegraph-check.sh -- --output json
```

## Likely Hot Spots

Check these areas before reaching for broad rewrites:

- Scanner traversal and ignore filtering.
- Path allocation, canonicalization, metadata calls, and sorting.
- File reading and SHA-256 hashing.
- Tree-sitter parse and Rust fact extraction.
- Fact-cache hit rate, read cost, write cost, and schema fingerprint invalidation.
- Datafox relation joins, string predicates, negation, and relation materialization.
- Output rendering, JSON serialization, stdout locking, and result aggregation.
- Channel contention, mutexed collectors, and worker imbalance.

## Optimization Rules

Keep scanner and check behavior streaming and bounded. Do not collect every path or every result unless the caller explicitly asks for deterministic aggregate presentation.

Push filtering as early as possible: ignore files, generated directories, cache directories, hidden paths, file size limits, language extensions, and binary checks.

Prefer sharded or per-worker state over one mutexed vector on hot paths. Merge only at output time or summary time.

For cache changes, verify both hit speed and miss speed. A faster cache hit path is not enough if misses regress cold-codebase runs heavily.

For Datafox changes, add focused rule fixtures or query tests before optimizing operators or joins. Preserve deterministic generated artifacts.
