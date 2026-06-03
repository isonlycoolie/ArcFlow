#!/usr/bin/env bash
# Install repo git hooks (prepare-commit-msg + pre-push for protected branches).

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [[ ! -d "$ROOT/.git" ]]; then
  echo "ERROR: not a git repository: $ROOT"
  exit 1
fi

install_hook() {
  local name="$1"
  cp "$ROOT/scripts/hooks/$name" "$ROOT/.git/hooks/$name"
  chmod +x "$ROOT/.git/hooks/$name"
  echo "Installed $name -> .git/hooks/$name"
}

install_hook prepare-commit-msg
install_hook pre-push
echo "See .github/BRANCH_POLICY.md for development/master workflow."
