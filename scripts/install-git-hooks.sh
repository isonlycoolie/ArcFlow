#!/usr/bin/env bash
# Install repo git hooks into .git/hooks (hook sources are not tracked in git).

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [[ ! -d "$ROOT/.git" ]]; then
  echo "ERROR: not a git repository: $ROOT"
  exit 1
fi

HOOKS_DIR="$ROOT/.git/hooks"

write_prepare_commit_msg() {
  cat >"$HOOKS_DIR/prepare-commit-msg" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
msg_file="${1:?commit message file required}"
tmp="${msg_file}.arcflow-hook.$$"
grep -v '^Co-authored-by: Cursor <cursoragent@cursor.com>$' "$msg_file" >"$tmp" || true
mv "$tmp" "$msg_file"
EOF
  chmod +x "$HOOKS_DIR/prepare-commit-msg"
  echo "Installed prepare-commit-msg -> .git/hooks/prepare-commit-msg"
}

write_pre_push() {
  cat >"$HOOKS_DIR/pre-push" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
protected='^refs/heads/(master|development)$'
while read -r _local_ref _local_sha remote_ref _remote_sha; do
  [[ -z "${remote_ref:-}" ]] && continue
  if [[ "$remote_ref" =~ $protected ]]; then
    branch="${remote_ref#refs/heads/}"
    echo "ERROR: Direct push to '${branch}' is not allowed."
    echo "Open a pull request instead. See CONTRIBUTING.md and .github/BRANCH_POLICY.md"
    exit 1
  fi
done
exit 0
EOF
  chmod +x "$HOOKS_DIR/pre-push"
  echo "Installed pre-push -> .git/hooks/pre-push"
}

write_prepare_commit_msg
write_pre_push
echo "See .github/BRANCH_POLICY.md for development/master workflow."
