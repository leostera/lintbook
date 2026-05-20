# lintbook

`lintbook` is a tree-sitter based lint runner built around project-local rule books. Users author rule descriptions in `.lintbook/rules`, agents generate Datafox queries in `.lintbook/gen`, and `lintbook compile` turns them into deterministic generated artifacts for `lintbook check`.

## Flow

```sh
curl https://get.lintbook.sh | sh -
lintbook setup
lintbook add "flag dbg! macro calls in committed Rust code"
lintbook compile --agent codex
lintbook check
lintbook check --json
```

`lintbook setup` creates:

- `lintbook.toml`
- `.lintbook/rules/template.md`
- `.lintbook/gen/.gitkeep`

The setup command prints manual MCP configuration snippets. It does not edit global or project MCP config files.

`lintbook` also ships embedded built-in rules compiled from Markdown intent and Datafox assets. Use `lintbook lints` to inspect the current built-in set.

`lintbook check` streams file results as they finish. Use `lintbook check --json` for newline-delimited JSON events, or `lintbook check --output json` when a single aggregate JSON document is needed.

## Custom Rules

Custom rules are Rust-only for the current path. A rule is a same-stem pair split by ownership:

- `.lintbook/rules/<name>.md` with minimal frontmatter and prose
- `.lintbook/gen/<name>.df` with the generated Datafox query set

Example frontmatter:

```yaml
---
id: rust.no-dbg
lang: rust
---
```

The Markdown body is the human rule description and becomes the default violation message.

Example query:

```text
node(Node, "macro_invocation", _, _, _, _), text(Node, Text), contains(Text, "dbg!")
```

Multiple queries can be separated with `;` in the same `.df` file when one rule has several match patterns.

Run `lintbook compile` after editing active rules. Generated files under `.lintbook/gen` are intended to be committed so `lintbook check` stays fast and agent-free.

`lintbook compile` is deterministic: it validates existing `.lintbook/gen/*.df` files and writes `.lintbook/gen/*.json`. If a real rule is missing its generated query, it fails and prints the missing path. Use `lintbook compile --agent codex` during authoring to ask Codex to generate missing `.df` files first, then compile them.

## Development

```sh
cargo fmt --all -- --check
cargo test --workspace --all-targets
cargo run -p lintbook-cli -- setup
```
