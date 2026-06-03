# Contributing to ArcFlow

Maintainers preparing a **public** release should complete [`OPEN_SOURCE.md`](OPEN_SOURCE.md) first.

## Repository map

Key top-level folders and their purpose:

- `runtime/`, Rust engine (`arcflow-core`) and server binaries
- `documentation/`, Full user, operator, and integrator docs (monorepo source of truth)
- `contracts/`, Normative wire formats and operator guides
- `docker/`, Compose stacks for local dev and self-hosting, see `docker/README.md`
- `scripts/`, CI, release, and operator helpers, see `scripts/README.md`
- `sdk-python/` and `sdk-typescript/`, Language SDKs

Not tracked in this repository: `docs/` (internal ADR pipeline), `webapp/` (separate ArcFlow-WebApp repo), sprint working plans, `.cursor/`, and local secrets such as `scripts/.pypi-token`.

Public documentation at https://arcflows.vercel.com is exported from a curated subset of `documentation/` in the WebApp repository (until `arcflow.dev` is configured).

## Commits

- **One task, one commit**, each commit should be the smallest independently testable change.
- **Subject line**, imperative mood, about five words (e.g. `test(sdk): add qdrant infra failure test`).
- **No bundled concerns**, do not mix runtime wiring, SDK exports, and tests in a single commit unless they are inseparable.
- **Authorship**, do not add `Co-authored-by: Cursor` (or other agent trailers). Run `bash scripts/install-git-hooks.sh` once per clone (installs commit-msg and pre-push hooks).

> <p style="color:red"><strong>One engine, every surface.</strong> Orchestration lives in arcflow-core (Rust). SDKs serialize workflow definitions into the Runtime Contract Specification (RCS), invoke the engine, and deserialize results. A fix in retry policy or recovery ships once and applies everywhere. <br><strong>Warning:</strong> keep commits focused, a single commit must not exceed 200 insertions; split unrelated changes into separate branches; never commit directly to `master` or `development`.</p>

## Branching model

ArcFlow uses a protected two-branch flow. See [`.github/BRANCH_POLICY.md`](.github/BRANCH_POLICY.md) for maintainer setup.

Key branches:

- **`development`**, default integration branch, all PRs target here first
- **`master`**, production / releases, PRs are merged from `development` only
- **feature/**, short-lived branches off `development` for individual work

Never push directly to `development` or `master`. The pre-push hook blocks this locally; GitHub branch protection is authoritative.

Typical flow:

1. `git checkout development && git pull && git checkout -b feature/my-change`
2. Open PR → `development` (fast CI / CI Docs must pass)
3. Release promotion: open PR `development` → `master` only after CI Full succeeded on the PR head commit

**CI Full** runs daily at **06:00 UTC** on `development`. If today's run finished before your commits landed, wait for the next scheduled run or a maintainer dispatches **Actions → CI Full → Run workflow** on `development` (manual bypass).

## Before pushing

From the repo root (Git Bash or WSL on Windows; or `.\scripts\ci-local.ps1` in PowerShell):

```bash
bash scripts/ci-smoke.sh    # fast gate (~1 min, no Rust compile)
bash scripts/ci-local.sh    # full fast PR mirror (includes smoke + Rust)
```

**CI smoke** runs on every PR via [`.github/workflows/ci-smoke.yml`](.github/workflows/ci-smoke.yml) and should turn green within about a minute.

This mirrors **fast PR checks** in [`.github/workflows/ci.yml`](.github/workflows/ci.yml): format, clippy, tests, commit size, secrets scan, contracts, documentation prose, static analysis gates, and provider credential audit.

Before merging **`development` → `master`**, ensure **CI Full** has passed on the PR head SHA (daily schedule or manual dispatch):

```bash
bash scripts/ci-local-full.sh
```

Or **Actions → CI Full → Run workflow** on `development`. The **Release promotion gate** on master PRs verifies a successful CI Full run on the exact commit.

Doc-only PRs trigger [`.github/workflows/ci-docs.yml`](.github/workflows/ci-docs.yml) instead of the Rust matrix. When editing `documentation/`, also run in the WebApp repo: `cd webapp && npm run docs:lint` (after export).

Individual gates: `bash scripts/check-no-unwrap.sh`, `bash scripts/check-no-sql-interpolation.sh`, `bash scripts/check-commit-size.sh`.

## Contribution areas, gaps, and suggested work

The project tracks several deferred or alpha features in the maturity matrix. Below are prioritized, contributor-friendly tasks that close practical gaps, grouped by impact and suggested difficulty. Pick an item, open an issue if none exists, and reference the issue in your PR. Follow the repository commit and branching rules above.

- **Graph recovery resume** (High impact, medium difficulty)
	- Gap: resuming a failed graph from persisted checkpoints is partial; dispatch logic is incomplete.
	- Suggested work: trace the resume codepath, add unit tests for checkpoint state, implement and validate the resume dispatcher, and add an end-to-end test that simulates a mid-graph crash and resume.
	- Labels: `area:runtime`, `good-first-issue` (small diagnostic tasks), `help-wanted` (implementation).

- **Server SSE streaming (`/v1/runs/{id}/events`)** (Medium impact, medium difficulty)
	- Gap: server SSE endpoint is deferred; SDKs and browser integrations currently poll or use in-process streaming.
	- Suggested work: design SSE contract (backwards compatible), implement server-side event marshaling, add integration tests for long-running runs, and update SDKs to consume SSE where applicable.
	- Labels: `area:server`, `help-wanted`.

- **`arcflow validate` CLI** (Medium impact, low difficulty)
	- Gap: CLI validate is a stub; CI uses JSON Schema instead.
	- Suggested work: wire CLI validation to the normative schema, add helpful error output and examples, and add acceptance tests to the CLI matrix.
	- Labels: `area:cli`, `good-first-issue`.

- **Operator dashboard (OSS parity / admin UX)** (High impact, high difficulty)
	- Gap: the production dashboard UI is in a private repo; OSS consumers rely on admin API and scripts.
	- Suggested work: audit admin API coverage, implement missing admin endpoints where reasonable, add example operator runbooks and SQL queries, and collaborate on a reference OSS dashboard implementation or a minimal admin UI shipped in `deploy/`.
	- Labels: `area:admin`, `area:ui`, `design-needed`.

- **OpenTelemetry metrics (stabilize alpha)** (Medium impact, medium difficulty)
	- Gap: OTel metrics are alpha; label cardinality and export surface need review.
	- Suggested work: add metrics unit tests, document recommended labels and sampling, add an optional CI job to lint metric definitions, and publish an example Prometheus/Grafana dashboard config under `deploy/`.
	- Labels: `area:observability`, `help-wanted`.

- **Edge WASM parity** (Low/medium impact, medium difficulty)
	- Gap: WASM supports linear stubs only; graph and RAG are unsupported.
	- Suggested work: document exact limitations, add tests that exercise the boundary, and prototype a subset of graph features that can be compiled to WASM or provide a clear compatibility shim.
	- Labels: `area:wasm`, `research`.

- **SDK parity and examples** (High impact, low/medium difficulty)
	- Gap: parity matrices exist; some examples and smoke tests are missing or unstable across SDKs.
	- Suggested work: add cross-SDK integration tests, small example apps for common patterns (RAG chat, published workflows), and keep the parity matrix updated in `sdks/`.
	- Labels: `area:sdk`, `good-first-issue`.

- **Security and hardening** (High impact, medium difficulty)
	- Gap: some operational patterns (label cardinality, admin key rotation, webhook HMAC) require additional docs and automated checks.
	- Suggested work: add a security checklist in `security/`, automate webhook HMAC verification tests, and add a simple key rotation example for `ARCFLOW_ADMIN_API_KEY` and runtime keys.
	- Labels: `area:security`, `help-wanted`.

Quick start for contributors:

1. Find or open an issue for the task you want to work on. If you open a new issue, include a short design or test plan and tag it with one of the labels above.
2. Branch from `development`: `git checkout development && git pull && git checkout -b feature/your-task`.
3. Keep changes focused: separate refactor from feature work (one commit per concern). Follow the commit message guidelines in this file.
4. Add tests and CI smoke commands where relevant. For runtime changes, include unit tests and an integration test that exercises the new behavior.
5. Open a PR to `development`, include a short description of the change, the testing steps, and the issue number.

If you want, I can create a starter issue for any of the suggested tasks and include a minimal test plan and checklist to accelerate contributions.
