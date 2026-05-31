#!/usr/bin/env bash
# Lightweight smoke check: SDK symbols and server routes referenced in documentation/ exist in repo.
set -euo pipefail
cd "$(dirname "$0")/.."
DOC_ROOT="documentation"
FAIL=0

check_grep() {
  local pattern="$1"
  local path="$2"
  local label="$3"
  if ! rg -q "$pattern" "$path" 2>/dev/null; then
    echo "MISSING: $label ($pattern in $path)"
    FAIL=1
  fi
}

# Documentation tree exists
if [[ ! -d "$DOC_ROOT" ]]; then
  echo "MISSING: $DOC_ROOT directory"
  exit 1
fi

COUNT=$(find "$DOC_ROOT" -name '*.md' | wc -l | tr -d ' ')
echo "Found $COUNT markdown files under $DOC_ROOT"
if [[ "$COUNT" -lt 100 ]]; then
  echo "WARN: expected ~118 documentation pages"
fi

# Core SDK exports documented
for sym in Agent Workflow TraceResult OpenAI Anthropic Gemini; do
  check_grep "$sym" sdk-python/arcflow "__all__ export $sym"
done

# Server routes documented in capabilities-aligned docs
for route in 'POST /v1/runs' 'GET /v1/runs' '/health' '/ready'; do
  if ! rg -q "$route" "$DOC_ROOT/server" 2>/dev/null; then
    echo "MISSING: server doc reference to $route"
    FAIL=1
  fi
done

# No em dash in documentation prose
if rg -U '[\x{2014}]' "$DOC_ROOT" 2>/dev/null; then
  echo "FAIL: em dash found in documentation/"
  FAIL=1
fi

# No markdown horizontal rules between sections
if rg '^---$' "$DOC_ROOT" 2>/dev/null; then
  echo "FAIL: horizontal rule (---) found in documentation/"
  FAIL=1
fi

if [[ "$FAIL" -eq 0 ]]; then
  echo "validate_documentation_claims: OK"
  exit 0
fi
exit 1
