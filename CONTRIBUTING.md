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
- **Authorship**, do not add `Co-authored-by: Cursor` (or other agent trailers). Run `bash scripts/install-git-hooks.sh` once per clone to strip them automatically.

## Before pushing

From the repo root:

```bash
bash scripts/ci-local.sh
```

Static analysis gates: `bash scripts/check-no-unwrap.sh` and `bash scripts/check-no-sql-interpolation.sh`.
