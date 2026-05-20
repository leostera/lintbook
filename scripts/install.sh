#!/bin/sh
set -eu

repo_url="${LINTBOOK_REPO_URL:-https://github.com/leostera/lintbook}"
local_path="${LINTBOOK_LOCAL_PATH:-}"

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo is required to install lintbook from source." >&2
  echo "Install Rust from https://rustup.rs, then run this installer again." >&2
  exit 1
fi

if [ -n "$local_path" ]; then
  cargo install --path "$local_path/crates/lintbook-cli" --bin lintbook --locked
else
  cargo install --git "$repo_url" --package lintbook-cli --bin lintbook --locked
fi

cat <<'JSON'

lintbook installed.

Manual MCP configuration:
{
  "mcpServers": {
    "lintbook": {
      "command": "lintbook",
      "args": ["mcp"]
    }
  }
}
JSON
