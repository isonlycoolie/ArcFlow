#!/usr/bin/env bash
# Reject commits that add more than MAX_CODE_LINES lines in code paths.

set -euo pipefail

MAX_CODE_LINES=100

# Extensions treated as "code" for the line budget (not markdown / sprint plans).
code_path() {
  local f="$1"
  [[ "$f" =~ \.(rs|json|toml|ya?ml|sh)$ ]] || return 1
  case "$f" in
    Cargo.lock|docs/*|arcflow_sprint*) return 1 ;;
    *package-lock.json) return 1 ;;
    *) return 0 ;;
  esac
}

count_added_lines() {
  local commit="$1"
  local total=0
  local add _del file
  while read -r add _del file; do
    [[ -z "${add:-}" || "$add" == "-" ]] && continue
    code_path "$file" || continue
    total=$((total + add))
  done < <(git diff-tree --no-commit-id --numstat -r "$commit")
  echo "$total"
}

if [[ "${1:-}" == "--commit" ]]; then
  commit="${2:?commit sha required}"
  n=$(count_added_lines "$commit")
  if [[ "$n" -gt "$MAX_CODE_LINES" ]]; then
    echo "ERROR: commit ${commit:0:12} adds ${n} code lines (max ${MAX_CODE_LINES})"
    git show --stat --oneline "$commit" | head -20
    exit 1
  fi
  echo "OK: commit ${commit:0:12} adds ${n} code lines"
  exit 0
fi

# PR / branch mode: check all commits not on base
BASE_REF="${GITHUB_BASE_REF:-main}"
RANGE=""
if git rev-parse "origin/${BASE_REF}" >/dev/null 2>&1; then
  RANGE="origin/${BASE_REF}..HEAD"
elif git rev-parse "origin/master" >/dev/null 2>&1; then
  RANGE="origin/master..HEAD"
elif [[ -n "${COMMIT_SIZE_BASE:-}" ]]; then
  RANGE="${COMMIT_SIZE_BASE}..HEAD"
fi

if [[ -n "$RANGE" ]]; then
  COMMITS=$(git rev-list --no-merges "$RANGE" 2>/dev/null || true)
else
  # Without a remote tracking branch, `git rev-list HEAD` walks the entire
  # history (false failures on legacy commits). Check only the tip commit
  # unless COMMIT_SIZE_BASE is set above.
  echo "Note: no origin/${BASE_REF} or origin/master; checking only HEAD (set COMMIT_SIZE_BASE=<sha> for a range)."
  COMMITS=$(git rev-list -n 1 HEAD 2>/dev/null || true)
fi
if [[ -z "$COMMITS" ]]; then
  echo "No commits to check in range ${RANGE}"
  exit 0
fi

FAILED=0
while read -r commit; do
  [[ -z "$commit" ]] && continue
  n=$(count_added_lines "$commit")
  if [[ "$n" -gt "$MAX_CODE_LINES" ]]; then
    echo "ERROR: commit ${commit:0:12} adds ${n} code lines (max ${MAX_CODE_LINES})"
    git log -1 --oneline "$commit"
    FAILED=1
  fi
done <<< "$COMMITS"

if [[ "$FAILED" -ne 0 ]]; then
  echo ""
  echo "Split commits so each adds at most ${MAX_CODE_LINES} lines of code."
  echo "Local check: bash scripts/check-commit-size.sh --commit <sha>"
  exit 1
fi

echo "All commits within ${MAX_CODE_LINES} code-line limit."
