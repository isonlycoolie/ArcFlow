#!/usr/bin/env bash
# Fail if unwrap() or expect() appear in arcflow-core library code outside #[cfg(test)].

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

FAILED=0
while IFS= read -r -d '' file; do
  test_lines=$(grep -n '^#\[cfg(test)\]' "$file" | head -1 | cut -d: -f1 || true)
  if [[ -n "$test_lines" ]]; then
    head -n "$((test_lines - 1))" "$file" | grep -nE '\.unwrap\(|\.expect\(' && FAILED=1 || true
  else
    grep -nE '\.unwrap\(|\.expect\(' "$file" && FAILED=1 || true
  fi
done < <(find runtime/arcflow-core/src -name '*.rs' -print0)

if [[ "$FAILED" -ne 0 ]]; then
  echo "ERROR: unwrap() or expect() found in non-test library code"
  exit 1
fi

echo "OK: no unwrap/expect in arcflow-core library paths"
