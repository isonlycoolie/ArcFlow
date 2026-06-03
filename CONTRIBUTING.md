# Contributing to ArcFlow

## Repository map

| Path | Purpose |
|------|---------|
| [`runtime/`](runtime/) | Rust engine (`arcflow-core`) and server binaries |
| [`documentation/`](documentation/) | Full user, operator, and integrator docs (monorepo source of truth) |
| [`contracts/`](contracts/) | Normative wire formats and operator guides |
| [`docker/`](docker/) | Compose stacks for local dev and self-hosting — see [`docker/README.md`](docker/README.md) |
| [`scripts/`](scripts/) | CI, release, and operator helpers — see [`scripts/README.md`](scripts/README.md) |
| [`dashboard/spec/`](dashboard/spec/) | Operator dashboard specification (UI implementation lives in the private ArcFlow-Dashboard repo) |
| [`sdk-python/`](sdk-python/) / [`sdk-typescript/`](sdk-typescript/) | Language SDKs |

**Not tracked in this repository:** `docs/` (internal ADR pipeline), `webapp/` (separate [ArcFlow-WebApp](https://github.com/isonlycoolie/ArcFlow-WebApp) repo), sprint working plans, `.cursor/`, and local secrets such as `scripts/.pypi-token`.

Public documentation at [arcflow.dev](https://arcflow.dev) is exported from a curated subset of `documentation/` in the WebApp repository.

## Commits

- **One task, one commit**, each commit should be the smallest independently testable change.
- **Subject line**, imperative mood, about five words (e.g. `test(sdk): add qdrant infra failure test`).
- **No bundled concerns**, do not mix runtime wiring, SDK exports, and tests in a single commit unless they are inseparable.
- **Authorship**, do not add `Co-authored-by: Cursor` (or other agent trailers). Run `bash scripts/install-git-hooks.sh` once per clone (installs commit-msg and pre-push hooks).

## Branching model

ArcFlow uses a protected two-branch flow. See [`.github/BRANCH_POLICY.md`](.github/BRANCH_POLICY.md) for maintainer setup.

| Branch | Role |
|--------|------|
| **`development`** | Default integration branch — **all PRs target here first** |
| **`master`** | Production / releases — **PRs from `development` only** |
| **feature/** | Your short-lived branch off `development` |

**Never push directly to `development` or `master`.** The pre-push hook blocks this locally; GitHub branch protection is authoritative.

1. `git checkout development && git pull && git checkout -b feature/my-change`
2. Open PR → **`development`** (fast **CI** / **CI Docs** must pass)
3. Release promotion: open PR **`development` → `master`** only after **CI Full** succeeded on the PR head commit

**CI Full** runs daily at **06:00 UTC** on `development`. If today's run finished before your commits landed, wait for the next scheduled run or a maintainer dispatches **Actions → CI Full → Run workflow** on `development` (manual bypass).

## Before pushing

From the repo root (Git Bash or WSL on Windows; or `.\scripts\ci-local.ps1` in PowerShell):

```bash
bash scripts/ci-local.sh
```

This mirrors **fast PR checks** in [`.github/workflows/ci.yml`](.github/workflows/ci.yml): format, clippy, tests, commit size, secrets scan, contracts, documentation prose, static analysis gates, and provider credential audit.

Before merging **`development` → `master`**, ensure **CI Full** has passed on the PR head SHA (daily schedule or manual dispatch):

```bash
bash scripts/ci-local-full.sh
```

Or **Actions → CI Full → Run workflow** on `development`. The **Release promotion gate** on master PRs verifies a successful CI Full run on the exact commit.

Doc-only PRs trigger [`.github/workflows/ci-docs.yml`](.github/workflows/ci-docs.yml) instead of the Rust matrix. When editing `documentation/`, also run in the WebApp repo: `cd webapp && npm run docs:lint` (after export).

Individual gates: `bash scripts/check-no-unwrap.sh`, `bash scripts/check-no-sql-interpolation.sh`, `bash scripts/check-commit-size.sh`.
