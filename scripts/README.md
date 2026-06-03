# Scripts

Helper scripts for contributors, maintainers, and operators. Run from the repository root unless noted.

## Contributor and CI

Two-tier CI: **fast** checks run automatically on every PR; **full** checks run via `workflow_dispatch` or locally before merge.

| Tier | GitHub workflow | Local script |
|------|-----------------|--------------|
| Fast (PR) | [`ci.yml`](../.github/workflows/ci.yml), [`ci-docs.yml`](../.github/workflows/ci-docs.yml) | [`ci-local.sh`](ci-local.sh) |
| Full (manual) | [`ci-full.yml`](../.github/workflows/ci-full.yml) | [`ci-local-full.sh`](ci-local-full.sh) |
| SDK Python (PR, path filter) | [`sdk-python.yml`](../.github/workflows/sdk-python.yml) | last section of `ci-local-full.sh` |
| SDK compat (weekly / manual) | [`sdk-python-compat.yml`](../.github/workflows/sdk-python-compat.yml) | — |
| Static e2e (PR path filter / manual) | [`static-e2e.yml`](../.github/workflows/static-e2e.yml) | `ci-local-full.sh` (Docker) |
| Integration memory (PR path filter / manual) | [`integration-memory.yml`](../.github/workflows/integration-memory.yml) | `ci-local-full.sh` (Postgres + Qdrant) |

| Script | Purpose |
|--------|---------|
| [`ci-local.sh`](ci-local.sh) | Mirror fast PR CI locally |
| [`ci-local-full.sh`](ci-local-full.sh) | Fast + trace bench, audit, rustdoc, TS build, optional Docker/Postgres |
| [`ci-local.ps1`](ci-local.ps1) | PowerShell wrapper (`-Full`, `-SkipPostgres`, etc.) |
| [`check-no-unwrap.sh`](check-no-unwrap.sh) | Fail on `.unwrap()` in production Rust paths |
| [`check-no-sql-interpolation.sh`](check-no-sql-interpolation.sh) | Fail on string-interpolated SQL |
| [`check-function-length.sh`](check-function-length.sh) | Function length gate |
| [`check-commit-size.sh`](check-commit-size.sh) | Commit size policy check |
| [`validate-rcs-schema.sh`](validate-rcs-schema.sh) | Validate RCS JSON Schema |
| [`verify-documentation-prose.mjs`](verify-documentation-prose.mjs) | Fail on internal planning language in `documentation/` |
| [`normalize-documentation-prose.mjs`](normalize-documentation-prose.mjs) | One-shot enterprise prose rewrite for `documentation/` |
| [`validate_documentation_claims.sh`](validate_documentation_claims.sh) | Cross-check doc claims (bash) |
| [`validate_documentation_claims.ps1`](validate_documentation_claims.ps1) | Cross-check doc claims (PowerShell) |
| [`install-git-hooks.sh`](install-git-hooks.sh) | Install prepare-commit-msg hook |
| [`assert_provider_no_credentials.py`](assert_provider_no_credentials.py) | Provider credential boundary test |
| [`assert_trace_overhead.py`](assert_trace_overhead.py) | Trace overhead smoke check |
| [`build-wasm.sh`](build-wasm.sh) | Build WASM alpha artifact |
| [`build_prebuilt_binaries.sh`](build_prebuilt_binaries.sh) | Build release binaries |

Before pushing, run:

```bash
bash scripts/ci-local.sh
```

Before merge to `master`, run full CI locally or dispatch **CI Full** in GitHub Actions:

```bash
bash scripts/ci-local-full.sh
```

## Release and maintainers

| Script | Purpose |
|--------|---------|
| [`build-python-sdk-wheels.sh`](build-python-sdk-wheels.sh) | Build Python SDK wheels for CI/release |
| [`verify-python-sdk-tag.sh`](verify-python-sdk-tag.sh) | Verify tag matches `sdk-python` version |
| [`check-pypi-version-absent.sh`](check-pypi-version-absent.sh) | Confirm version not already on PyPI |
| [`publish-pypi.ps1`](publish-pypi.ps1) | Build and upload with token from env or `scripts/.pypi-token` |
| [`publish-pypi-local.ps1`](publish-pypi-local.ps1) | Upload wheels from `sdk-python/dist/` |
| [`build-python-sdk.ps1`](build-python-sdk.ps1) | Windows wheel build into `sdk-python/dist/` |
| [`.pypi-token.example`](.pypi-token.example) | Template for gitignored `scripts/.pypi-token` |

## Operator helpers

These scripts call the admin API or smoke static-product flows against a **running** `arcflow-server`. They require configured API keys and network access to your stack.

| Script | Purpose |
|--------|---------|
| [`static-provision-site.sh`](static-provision-site.sh) | Provision a static product site |
| [`static-ingest-knowledge.sh`](static-ingest-knowledge.sh) | Ingest knowledge for a site |
| [`static-publish-chat.sh`](static-publish-chat.sh) | Publish chat workflow |
| [`static-smoke.sh`](static-smoke.sh) | End-to-end static product smoke |
| [`relay-provision-site.sh`](relay-provision-site.sh) | Relay site provisioning helper |
| [`load-test-runs.sh`](load-test-runs.sh) | Load test run creation |

See [documentation/deployment/overview.md](../documentation/deployment/overview.md) and [documentation/static-product/overview.md](../documentation/static-product/overview.md).
