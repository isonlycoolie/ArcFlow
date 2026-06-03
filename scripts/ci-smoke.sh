#!/usr/bin/env bash
# Fast PR gate (~1 min, no Rust compile). Mirrors .github/workflows/ci-smoke.yml.
# Run from repo root: bash scripts/ci-smoke.sh

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

run_step() {
  echo ""
  echo "=== $1 ==="
  shift
  "$@"
}

run_step "commit-size" bash scripts/check-commit-size.sh

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

run_step "documentation-prose" node scripts/verify-documentation-prose.mjs

run_step "no-unwrap" bash scripts/check-no-unwrap.sh

run_step "no-sql-interpolation" bash scripts/check-no-sql-interpolation.sh

run_step "function-length" bash scripts/check-function-length.sh

run_step "structure-check" bash -c '
  set -euo pipefail
  for dir in runtime/arcflow-core sdk-python sdk-typescript sdk-java sdk-go cli contracts; do
    test -d "$dir" || (echo "Missing required directory: $dir" && exit 1)
  done
  if [ -d docs ]; then echo "Note: local docs/ may exist but must not be tracked by git"; fi
  echo "OK: required directories present"
'

run_step "provider-security-audit" python scripts/assert_provider_no_credentials.py

echo ""
echo "ci-smoke: all fast gates passed"
echo "For full PR checks run: bash scripts/ci-local.sh"
