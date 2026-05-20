#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: flamegraph-check.sh [--root] [--freq N] [--output PATH] [--] [lintbook-check-args...]

Profiles `lintbook check` with cargo flamegraph from the repository root.
Default lintbook check args: --json

Examples:
  bash .agents/skills/lintbook-performance/scripts/flamegraph-check.sh
  bash .agents/skills/lintbook-performance/scripts/flamegraph-check.sh -- --output json
  bash .agents/skills/lintbook-performance/scripts/flamegraph-check.sh --root -- --json
USAGE
}

root_args=()
freq=997
timestamp="$(date +%Y%m%d-%H%M%S)"
output=".agents/profiles/lintbook-check-${timestamp}.svg"
lint_args=()

while [ "$#" -gt 0 ]; do
  case "$1" in
    --root)
      root_args=(--root)
      shift
      ;;
    --freq)
      if [ "$#" -lt 2 ]; then
        echo "error: --freq requires a value" >&2
        exit 2
      fi
      freq="$2"
      shift 2
      ;;
    --output|-o)
      if [ "$#" -lt 2 ]; then
        echo "error: --output requires a path" >&2
        exit 2
      fi
      output="$2"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    --)
      shift
      lint_args=("$@")
      break
      ;;
    *)
      lint_args+=("$1")
      shift
      ;;
  esac
done

if [ "${#lint_args[@]}" -eq 0 ]; then
  lint_args=(--json)
fi

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"
mkdir -p "$(dirname "$output")"

cargo flamegraph \
  --profile release \
  -p lintbook-cli \
  -b lintbook \
  -F "$freq" \
  --ignore-status \
  -o "$output" \
  --title "lintbook check" \
  --notes "repo=${repo_root} generated_at=$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  "${root_args[@]}" \
  -- check "${lint_args[@]}"

printf 'flamegraph: %s\n' "$output"
