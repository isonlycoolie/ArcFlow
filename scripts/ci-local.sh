#!/usr/bin/env bash
# Mirror .github/workflows/ci-smoke.yml + ci.yml. Run from repo root:
#   bash scripts/ci-local.sh

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

bash scripts/ci-smoke.sh

export CARGO_TERM_COLOR=always
export RUST_BACKTRACE=1

run_step() {
  echo ""
  echo "=== $1 ==="
  shift
  "$@"
}

run_step "format" cargo fmt --check
run_step "lint" cargo clippy --workspace --all-targets -- -D warnings
run_step "test" cargo test --workspace
run_step "documentation-prose" node scripts/verify-documentation-prose.mjs
run_step "no-unwrap" bash scripts/check-no-unwrap.sh
run_step "no-sql-interpolation" bash scripts/check-no-sql-interpolation.sh
run_step "function-length" bash scripts/check-function-length.sh

echo ""
echo "ci-local: all fast CI steps passed"
echo "Before merge to master, run: bash scripts/ci-local-full.sh (or Actions → CI Full)"
