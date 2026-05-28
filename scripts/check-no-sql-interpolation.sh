#!/usr/bin/env bash
# Fail if memory-backend SQL is built via format! or dynamic string concat (Sprint 4 gate).

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

SEARCH_ROOT="runtime/arcflow-core/src/memory"
FAILED=0

if git grep -nE 'sqlx::query!\s*\(\s*&\s*format!|sqlx::query\s*\(\s*&\s*format!|sqlx::query\s*\(\s*&\s*\(' -- "$SEARCH_ROOT" 2>/dev/null; then
  echo "ERROR: dynamic SQL string construction detected"
  FAILED=1
fi

if git grep -nE 'format!\s*\([^)]*(SELECT|INSERT|UPDATE|DELETE)' -- "$SEARCH_ROOT" 2>/dev/null; then
  echo "ERROR: format! used to build SQL in memory backends"
  FAILED=1
fi

if [[ "$FAILED" -ne 0 ]]; then
  exit 1
fi

echo "OK: memory SQL uses static query strings with binds (no interpolation)"
