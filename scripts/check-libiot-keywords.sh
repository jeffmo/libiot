#!/usr/bin/env bash
#
# Fails CI (and pre-commit) if any crate's Cargo.toml omits "libiot" from
# its keywords array. This keeps the workspace-wide keyword enforceable as
# new crates are added without requiring each author to remember the rule.
#
# Exit code:
#   0  every crate declares "libiot" in keywords
#   1  at least one crate is missing the keyword (details printed to stderr)

set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

fail=0

shopt -s nullglob
for toml in crates/*/Cargo.toml; do
    # Extract the keywords = [ ... ] array (possibly multi-line) and check for
    # "libiot". Using awk keeps this dependency-free — no jq, no python.
    if ! awk '
        BEGIN { in_keywords = 0; found = 0 }
        /^[[:space:]]*keywords[[:space:]]*=/ {
            in_keywords = 1
        }
        in_keywords {
            if (index($0, "\"libiot\"") > 0) { found = 1 }
            if (index($0, "]") > 0) { in_keywords = 0 }
        }
        END { exit !found }
    ' "$toml"; then
        echo "FAIL: $toml is missing \"libiot\" from its keywords array" >&2
        fail=1
    fi
done

if [[ "$fail" -eq 0 ]]; then
    echo "OK: every crate declares \"libiot\" in its keywords."
fi

exit "$fail"
