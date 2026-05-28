#!/usr/bin/env bash
# Install repo git hooks (prepare-commit-msg strips Cursor co-author trailers).

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HOOK_SRC="$ROOT/scripts/hooks/prepare-commit-msg"
HOOK_DST="$ROOT/.git/hooks/prepare-commit-msg"

if [[ ! -d "$ROOT/.git" ]]; then
  echo "ERROR: not a git repository: $ROOT"
  exit 1
fi

cp "$HOOK_SRC" "$HOOK_DST"
chmod +x "$HOOK_DST"
echo "Installed prepare-commit-msg -> .git/hooks/prepare-commit-msg"
