#!/usr/bin/env bash
# Apply GitHub branch protection for development + master (requires gh auth + admin).
# Usage: bash scripts/setup-github-branch-policy.sh [owner/repo]

set -euo pipefail

REPO="${1:-isonlycoolie/ArcFlow}"
OWNER="${REPO%%/*}"
NAME="${REPO#*/}"

if ! command -v gh &>/dev/null; then
  echo "ERROR: gh CLI required. Install from https://cli.github.com/ and run: gh auth login"
  exit 1
fi

apply_protection() {
  local branch="$1"
  local payload_file="$2"
  echo "Configuring protection for ${branch}..."
  gh api -X PUT "repos/${OWNER}/${NAME}/branches/${branch}/protection" --input "$payload_file"
}

DEV_PAYLOAD="$(mktemp)"
MASTER_PAYLOAD="$(mktemp)"
trap 'rm -f "$DEV_PAYLOAD" "$MASTER_PAYLOAD"' EXIT

cat >"$DEV_PAYLOAD" <<'EOF'
{
  "required_status_checks": {
    "strict": true,
    "contexts": [
      "Format check",
      "Clippy",
      "Tests",
      "Commit size (max 100 code lines)",
      "Secret pattern scan",
      "Validate RCS schema",
      "Documentation prose check",
      "No unwrap in library code",
      "No SQL string interpolation",
      "Function length check",
      "Directory structure",
      "Provider credential scan"
    ]
  },
  "enforce_admins": true,
  "required_pull_request_reviews": {
    "dismiss_stale_reviews": true,
    "required_approving_review_count": 1
  },
  "restrictions": null,
  "required_linear_history": false,
  "allow_force_pushes": false,
  "allow_deletions": false
}
EOF

cat >"$MASTER_PAYLOAD" <<'EOF'
{
  "required_status_checks": {
    "strict": true,
    "contexts": [
      "Trace overhead budget",
      "Security audit",
      "Rustdoc build",
      "TypeScript SDK build",
      "Postgres integration",
      "Release promotion gate"
    ]
  },
  "enforce_admins": true,
  "required_pull_request_reviews": {
    "dismiss_stale_reviews": true,
    "required_approving_review_count": 1
  },
  "restrictions": null,
  "required_linear_history": false,
  "allow_force_pushes": false,
  "allow_deletions": false
}
EOF

echo "Repository: ${REPO}"
echo "Ensure branch development exists on origin before continuing."
read -r -p "Continue? [y/N] " ans
[[ "${ans:-}" =~ ^[Yy]$ ]] || exit 0

apply_protection "development" "$DEV_PAYLOAD"
apply_protection "master" "$MASTER_PAYLOAD"

echo ""
echo "Done. Set default branch:"
echo "  gh api repos/${REPO} -X PATCH -f default_branch=development"
echo "See .github/BRANCH_POLICY.md for rulesets and CI Full policy."
