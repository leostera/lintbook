---
name: lintbook-performance
description: Profile and optimize lintbook performance in this repository. Use when working on `lintbook check` latency, cargo flamegraph runs, cold or warm fact-cache timing, scanner parallelism, Datafox query evaluation speed, output streaming overhead, or large-codebase performance regressions.
---

# Lintbook Performance

## Core Workflow

1. Start from a concrete performance question: cold cache, warm cache, output streaming, scanner traversal, fact extraction, or Datafox evaluation.
2. Record the current git state and benchmark command before changing code.
3. Build release code before timing: `cargo build --release -p lintbook-cli`.
4. Measure warm and cold runs separately. Only remove `.lintbook/cache` when explicitly measuring cold-cache behavior.
5. Profile with `cargo flamegraph` after establishing a baseline, not before.
6. Optimize one suspected bottleneck at a time, then re-run the same timing and flamegraph command.
7. Finish with numbers: before, after, command, cache state, and any remaining uncertainty.

## Commands

Use the checked-in helper for repeatable flamegraph runs:

```bash
bash .agents/skills/lintbook-performance/scripts/flamegraph-check.sh
```

Pass lintbook check arguments after `--`:

```bash
bash .agents/skills/lintbook-performance/scripts/flamegraph-check.sh -- --json
bash .agents/skills/lintbook-performance/scripts/flamegraph-check.sh -- crates/lintbook-rules/src/lib.rs
```

Use `--root` only when `cargo flamegraph` needs elevated sampling privileges and the user has approved that path:

```bash
bash .agents/skills/lintbook-performance/scripts/flamegraph-check.sh --root -- --json
```

For quick timing, prefer the release binary directly so Cargo overhead is not part of the measurement:

```bash
target/release/lintbook check --json
target/release/lintbook check --output json
target/release/lintbook check
```

## Interpretation

Read `references/flamegraph-workflow.md` when deciding where to spend optimization effort or when comparing before and after profiles.

Keep the hot path consistent with the repo guidance in `AGENTS.md`: unordered by default, bounded streaming, early filtering, no whole-repo path collection for primary check runs, and memory proportional to active workers rather than repository size.
