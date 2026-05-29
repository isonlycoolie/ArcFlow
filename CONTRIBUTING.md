# Contributing to ArcFlow

## Commits

- **One task, one commit** — each commit should be the smallest independently testable change.
- **Subject line** — imperative mood, about five words (e.g. `test(sdk): add qdrant infra failure test`).
- **No bundled concerns** — do not mix runtime wiring, SDK exports, and tests in a single commit unless they are inseparable.
- **Authorship** — do not add `Co-authored-by: Cursor` (or other agent trailers). Run `bash scripts/install-git-hooks.sh` once per clone to strip them automatically.

## Before pushing

From the repo root:

```bash
bash scripts/ci-local.sh
```

Static analysis gates: `bash scripts/check-no-unwrap.sh` and `bash scripts/check-no-sql-interpolation.sh`.
