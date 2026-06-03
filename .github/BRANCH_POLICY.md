# Branch policy (enterprise)

ArcFlow uses a two-branch integration model. GitHub branch protection is authoritative; local git hooks catch mistakes early.

## Branches

| Branch | Purpose | Direct push |
|--------|---------|-------------|
| **`development`** | Default integration branch, all feature work merges here | Blocked |
| **`master`** | Production / release line | Blocked |
| **feature/** | Short-lived contributor branches | Allowed (via PR only into `development`) |

## Contributor flow

1. Branch from **`development`**: `git checkout development && git pull && git checkout -b feature/my-change`
2. Run `bash scripts/ci-local.sh` locally
3. Open a **pull request → `development`** (fast **CI** or **CI Docs** must pass)
4. Maintainers promote **`development` → `master`** via PR only after **CI Full** passed on the PR head commit

> <p style="color:red"><strong>One engine, every surface.</strong> Orchestration lives in arcflow-core (Rust). SDKs serialize workflow definitions into the Runtime Contract Specification (RCS), invoke the engine, and deserialize results. A fix in retry policy or recovery ships once and applies everywhere. <br><strong>Warning:</strong> keep commits focused, a single commit must not exceed 200 insertions; split unrelated changes into separate branches; never commit directly to `master` or `development`.</p>

## CI Full (master promotion gate)

**CI Full** runs:

- **Daily** at **06:00 UTC** on the `development` tip (scheduled workflow)
- **Manually** via **Actions → CI Full → Run workflow** (bypass when daily run already passed or new commits need immediate verification)

A **`development` → `master`** PR cannot merge until:

1. Head branch is **`development`**
2. **Release promotion gate** passes
3. A **successful CI Full** workflow run exists for the **exact PR head SHA**
4. Branch protection required checks are green

If new commits land on `development` after today's scheduled CI Full, wait for the next daily run or dispatch CI Full manually on `development`.

## Required status checks (branch protection)

### `development` (fast PR CI)

Configure these check names from [`.github/workflows/ci-smoke.yml`](workflows/ci-smoke.yml), [`.github/workflows/ci.yml`](workflows/ci.yml), and [`.github/workflows/ci-docs.yml`](workflows/ci-docs.yml):

- **`CI smoke`** (always runs on every PR; sub-minute)
- `Format check`
- `Clippy`
- `Tests`
- `Commit size (max 100 code lines)`
- `Secret pattern scan`
- `Validate RCS schema`
- `Documentation prose check`
- `No unwrap in library code`
- `No SQL string interpolation`
- `Function length check`
- `Directory structure`
- `Provider credential scan`

Doc-only PRs may omit Rust jobs; require at least **CI Docs** jobs when applicable.

### `master` (release promotion)

From [`.github/workflows/ci-full.yml`](workflows/ci-full.yml) and [`.github/workflows/merge-gate-master.yml`](workflows/merge-gate-master.yml):

- `Trace overhead budget`
- `Security audit`
- `Rustdoc build`
- `TypeScript SDK build`
- `Postgres integration`
- `Release promotion gate`

## One-time GitHub setup (maintainers)

Requires `gh auth login` with admin access to the repository.

```bash
# 1. Create and push development from master (if not present)
git fetch origin
git checkout origin/master
git checkout -B development
git push -u origin development

# 2. Apply branch protection (edit OWNER/REPO if needed)
bash scripts/setup-github-branch-policy.sh

# 3. Set default branch to development (UI or gh)
gh api repos/isonlycoolie/ArcFlow -X PATCH -f default_branch=development
```

Optional: **Settings → Rules → Rulesets**, require PRs into `master` to use **`development`** as the head branch.

## Local hooks

```bash
bash scripts/install-git-hooks.sh
```

Installs `pre-push` (blocks direct push to `development` / `master`) and `prepare-commit-msg` (strips agent co-author trailers).
