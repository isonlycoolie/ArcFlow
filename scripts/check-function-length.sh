#!/usr/bin/env bash
# Fail when any function in arcflow-core/src exceeds MAX_FUNCTION_LINES (Sprint 2: 40).

set -euo pipefail

MAX_FUNCTION_LINES=40
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

FAILED=0
while IFS= read -r -d '' file; do
  test_start=$(grep -n '^#\[cfg(test)\]' "$file" | head -1 | cut -d: -f1 || true)
  if [[ -n "$test_start" ]]; then
    slice=$(head -n "$((test_start - 1))" "$file")
  else
    slice=$(cat "$file")
  fi
  while IFS= read -r line; do
    [[ "$line" =~ fn[[:space:]] ]] || continue
    start=${line%%:*}
    name=$(echo "$line" | sed -n 's/.*fn \([^(]*\).*/\1/p')
    count=0
    end=$((start + 1))
    total=$(echo "$slice" | wc -l)
    while [[ "$end" -le "$total" ]]; do
      body=$(echo "$slice" | sed -n "${end}p")
      if echo "$line $body" | grep -q '{'; then
        break
      fi
      end=$((end + 1))
    done
    end=$((end + 1))
    while [[ "$end" -le "$total" ]]; do
      body=$(echo "$slice" | sed -n "${end}p")
      count=$((count + 1))
      if echo "$body" | grep -qE '^[[:space:]]*\}'; then
        break
      fi
      end=$((end + 1))
    done
    if [[ "$count" -gt "$MAX_FUNCTION_LINES" ]]; then
      echo "ERROR: $file fn $name is $count lines (max $MAX_FUNCTION_LINES)"
      FAILED=1
    fi
  done < <(echo "$slice" | grep -n 'fn ')
done < <(find runtime/arcflow-core/src -name '*.rs' -print0)

if [[ "$FAILED" -ne 0 ]]; then
  exit 1
fi

echo "OK: all functions within ${MAX_FUNCTION_LINES} lines"
