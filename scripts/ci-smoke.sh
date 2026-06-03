#!/usr/bin/env bash
# Fast CI smoke checks (~1 min, no Rust compile). Mirror .github/workflows/ci-smoke.yml
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
run_step() { echo ""; echo "=== $1 ==="; shift; "$@"; }
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
run_step "structure-check" bash -c '
  set -euo pipefail
  for dir in runtime/arcflow-core sdk-python sdk-typescript sdk-java sdk-go cli contracts; do
    test -d "$dir" || (echo "Missing required directory: $dir" && exit 1)
  done
  echo "OK: required directories present"
'
run_step "commit-size" bash scripts/check-commit-size.sh
run_step "validate-contracts" bash scripts/validate-rcs-schema.sh
run_step "provider-security-audit" python3 scripts/assert_provider_no_credentials.py
echo ""; echo "ci-smoke: all steps passed"
