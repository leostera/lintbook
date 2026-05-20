# AGENTS.md

This file provides guidance for coding agents working with this repository.

# lintbook

## Project Overview

`lintbook` is a tree-sitter based lint runner for project-local rule books. The active product path is Markdown rule descriptions in `.lintbook/rules` plus generated Datafox queries and compiled artifacts in `.lintbook/gen`.

## Architecture

- `crates/lintbook-cli` provides the `lintbook` binary.
- `crates/lintbook-rules` owns `.lintbook/rules` parsing, `.lintbook/gen` compilation, Rust tree-sitter fact extraction, and Datafox evaluation.
- `../datafox` is the sibling Datalog parser and streaming query engine crate used through a path dependency.
- `crates/lintbook-lang` owns grammar detection, parser wiring, and AST dumping.
- `crates/lintbook-config`, `lintbook-core`, and `lintbook-scanner` provide shared config, result, and file scanning types.
- `crates/lintbook-mcp` exposes MCP guidance and compile tooling.
- The old Rust language rule crate has been removed after conversion to built-in Datafox assets; the remaining parked language-specific crates are intentionally not workspace members or runtime dependencies.

## Key Commands

- `cargo fmt --all -- --check`
- `cargo test --workspace --all-targets`
- `cargo run -p lintbook-cli -- setup`
- `cargo run -p lintbook-cli -- compile`
- `cargo run -p lintbook-cli -- check`
- `cargo run -p lintbook-cli -- dump-ast --lang rust path/to/file.rs`

## Development Notes

- Keep the primary path focused on `.lintbook/rules/*.md` + `.lintbook/gen/*.df` to `.lintbook/gen/*.json`.
- Generated rule artifacts are deterministic and intended to be committed.
- Keep plain `lintbook compile` deterministic and agent-free. Agent generation is opt-in through `lintbook compile --agent codex`.
- `lintbook check` should fail when active source rules are newer than generated artifacts; it should not auto-compile.
- MCP setup is manual-only. Do not edit global or project MCP config files from setup or the installer.
- Update insta snapshots in `crates/lintbook-tests/tests/snapshots` only after confirming CLI output changes are expected.

## Large-Codebase File Streaming Notes

When revisiting scanner performance, design the file path as a bounded stream for monorepos and very large repositories.

- Use BurntSushi's `ignore` walker for parallel traversal and apply glob filtering with the intended globdir/glob-style crate at the walker/filter layer, not after collecting every path.
- Keep the hot path unordered by default. Do not force global sorting, deterministic traversal, or a single serialized collector unless the caller explicitly asks for deterministic output.
- Avoid `Vec<PathBuf>` as the scanner boundary for the primary check path. Prefer a bounded channel or pull-based iterator that lets workers parse/evaluate files while traversal continues.
- Avoid one mutexed result vector for every file. Prefer per-worker batches, bounded channels, or sharded collectors that merge only at output time.
- Do not run separate full-tree passes for language detection and linting on large repos. Reuse the same traversal stream, or cache detected extensions/project metadata from the first pass.
- Push filtering as early as possible: hidden rules, ignore files, explicit include/exclude globs, language extensions, generated/cache directories, size limits, and binary-file checks should happen before reading file contents.
- Keep paths cheap. Avoid canonicalizing every file, avoid unnecessary metadata calls, and use absolute repo-root traversal only when the walker can provide it without per-file filesystem round trips.
- Make memory usage proportional to active workers, not repository size. File contents, tree-sitter trees, facts, and Datafox relations should be dropped or cached per file as soon as that file is evaluated.
- Preserve backpressure. If parsing or rule evaluation is slower than walking, traversal should naturally slow down instead of buffering millions of paths.
- For machine output, consider streaming JSON/NDJSON events for very large runs; only aggregate summaries that are cheap and bounded.
- Keep deterministic output as a presentation-layer option. If needed, sort only the final bounded violation set, not all visited files.
