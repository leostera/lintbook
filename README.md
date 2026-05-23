# lintbook

`lintbook` is an LLM-powered linter for repeatable quality checks that are too project-specific, contextual, or new to wait for a general-purpose lint rule.

You write the intent of a rule in plain Markdown, let an agent translate it into a Datafox query once, and then run that query locally at lint speed. The LLM is in the authoring path, not the checking path, so `lintbook check` stays deterministic, cacheable, and CI-friendly.

## Why lintbook

Agents are good at noticing issues, but they are a poor runtime for checks you want on every commit:

- They are nondeterministic and can miss the same issue on the next run.
- They are slow; one model call per file or file group does not scale.
- They are expensive; repeated commit-time checks burn tokens quickly.

Traditional linters solve those runtime problems, but writing custom rules for different lint engines takes time, tool-specific knowledge, and maintenance energy. Most linters are excellent at enforcing rules that already exist. They are weaker when a team wants to encode local judgment:

- "Do not use this internal helper outside the API boundary."
- "Public Rust types in this crate need documentation."
- "Avoid this Python import pattern because our runtime loader cannot see it."
- "Flag this migration shape unless it is paired with this rollback statement."

`lintbook` is built for those rules. The rulebook lives in your repository, the generated query artifacts are committed, and the fast path runs without a model, network call, or agent session.

The result is:

- Deterministic: rules compile to Datafox queries.
- Fast: a roughly 30k line project can run in about 300ms.
- Cheap: tokens are used only while compiling or authoring lints.

## Install

```sh
brew install leostera/lintbook/lintbook
```

or:

```sh
curl https://get.lintbook.sh | sh -
```

## Quick Start

Set up lintbook in a repository:

```sh
lintbook setup
```

This creates:

- `lintbook.toml`
- `.lintbook/rules/template.md`
- `.lintbook/gen/.gitkeep`

It also prints MCP configuration snippets for agents that can use lintbook's rule-authoring tools. `lintbook setup` does not edit global or project MCP config files automatically.

Add a rule description:

```sh
lintbook add "flag dbg! macro calls in committed Rust code"
```

Generate the missing Datafox query with an agent, then compile it:

```sh
lintbook compile --agent codex
```

Run the linter:

```sh
lintbook check
```

Check a smaller target:

```sh
lintbook check crates/lintbook-cli/src/main.rs
lintbook check crates/lintbook-rules
lintbook check "crates/**/*.rs"
```

Stream machine-readable output:

```sh
lintbook check --json
```

List the built-in rules shipped with the binary:

```sh
lintbook lints
lintbook lints --output json
```

## Rule Files

A project rule starts as a Markdown file in `.lintbook/rules`:

```markdown
---
id: rust.no-dbg
lang: rust
---

We do not want dbg! macro calls in production code.
```

The Markdown body is the human intent. It becomes the default violation message unless the generated query supplies more specific output.

The generated query belongs in `.lintbook/gen` with the same rule stem:

```text
node(Node, "macro_invocation", _, _, _, _), text(Node, Text), contains(Text, "dbg!")
```

`lintbook compile` validates the Markdown and Datafox files, then writes deterministic compiled artifacts in `.lintbook/gen/*.json`. Commit the Markdown, Datafox, and compiled JSON files so `lintbook check` can run quickly in CI and on other machines.

Plain `lintbook compile` is deterministic and agent-free. If a Markdown rule is missing its `.df` query, it reports the missing generated file. Agent generation is opt-in through `lintbook compile --agent codex`.

## How It Works

`lintbook` separates rule authoring from rule execution.

During authoring, a person writes the rule intent in Markdown. An agent can use lintbook's MCP guidance to inspect the available facts, Datafox syntax, and examples, then generate a matching `.df` query. The agent's job ends once the query compiles successfully; it should not try to rewrite the codebase to satisfy the new rule.

During compilation, lintbook reads `.lintbook/rules/*.md` and same-stem `.lintbook/gen/*.df` files. It validates frontmatter, parses the Datafox query, checks that the rule can be prepared, and writes a compact `.json` artifact. These compiled artifacts are stable build products, not temporary cache files.

During checking, lintbook scans the requested files, detects supported languages, parses each file with tree-sitter, and extracts a fact set from the syntax tree. Facts describe nodes, text, parent-child structure, field names, descendants, line numbers, and language-specific helper relations used by the built-in and generated rules.

Those extracted facts are cached by file content hash and required predicate set. If a file has the same SHA-256 and the active rules need the same facts, lintbook reloads the cached facts instead of reparsing and re-emitting the tree-sitter fact graph.

Datafox then evaluates the prepared rule queries over each file's facts. Planning is kept hot through prepared query caches, and results are streamed as files complete. Human output is meant for local use; `lintbook check --json` emits newline-delimited JSON events for editors, CI, and large repositories.

The important boundary is that `lintbook check` never asks an LLM what to do. The LLM helps create the rule. The committed rule artifacts make the runtime deterministic, local, and fast.
