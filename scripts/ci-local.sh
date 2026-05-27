#!/usr/bin/env bash
# Mirror .github/workflows/ci.yml jobs that run on ubuntu-latest (no commit-size).
# Run from repo root: bash scripts/ci-local.sh

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

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

if ! command -v cargo-audit &>/dev/null; then
  echo "Installing cargo-audit..."
  cargo install cargo-audit
fi
run_step "audit" cargo audit

run_step "doc" env RUSTDOCFLAGS=-D warnings cargo doc --workspace --no-deps

run_step "no-unwrap" bash scripts/check-no-unwrap.sh

run_step "function-length" bash scripts/check-function-length.sh

run_step "secrets-scan" bash -c '
  set -euo pipefail
  if git grep -nE "sk-[A-Za-z0-9]{20,}|AKIA[0-9A-Z]{16}" -- . ":(exclude)target" ":(exclude).git" 2>/dev/null; then
    echo "ERROR: Possible API key or AWS access key id pattern in tree"
    exit 1
  fi
  if git grep -nE "BEGIN (RSA|OPENSSH|EC) PRIVATE KEY" -- . ":(exclude)target" ":(exclude).git" 2>/dev/null; then
    echo "ERROR: Possible PEM private key material in tree"
    exit 1
  fi
  echo "OK: no blocked secret-like patterns"
'

run_step "validate-contracts" bash scripts/validate-rcs-schema.sh

run_step "structure-check" bash -c '
  set -euo pipefail
  for dir in runtime/arcflow-core sdk-python sdk-typescript sdk-java sdk-go cli contracts; do
    test -d "$dir" || (echo "Missing required directory: $dir" && exit 1)
  done
  if [ -d docs ]; then echo "Note: local docs/ may exist but must not be tracked by git"; fi
  echo "OK: required directories present"
'

echo ""
echo "ci-local: all steps passed"
