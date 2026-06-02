# Python SDK release (maintainers)

This runbook covers PyPI trusted publishing, GitHub environment setup, and cutting a release for the `arcflow` package.

## Dual-pipeline CI

Three workflows share responsibility so PRs stay fast and releases stay complete.

| Workflow | When it runs | Purpose |
|----------|----------------|---------|
| [SDK Python](https://github.com/isonlycoolie/ArcFlow/actions/workflows/sdk-python.yml) | Every PR (path-filtered) and push to `master` | **Fast:** Ubuntu, Python 3.11, `maturin develop`, lint, pytest (~3 min) |
| [SDK Python Compat](https://github.com/isonlycoolie/ArcFlow/actions/workflows/sdk-python-compat.yml) | Push to `master`, weekly schedule, manual dispatch | **Compat:** 3 OS × Python 3.9–3.12, release wheels |
| [Publish Python SDK](https://github.com/isonlycoolie/ArcFlow/actions/workflows/publish-python-sdk.yml) | Tag `sdk-python/v*` only | **Release:** cibuildwheel, PyPI OIDC, GitHub Release |

```text
PR / feature branch  -->  SDK Python (fast)
merge to master      -->  SDK Python Compat (full matrix)
tag sdk-python/v*    -->  Publish Python SDK (cibuildwheel + PyPI)
```

### Branch protection (recommended)

- **Pull requests:** require status check **SDK Python** / `Fast (Ubuntu, Python 3.11)`.
- **`master`:** require **SDK Python Compat** (or monitor it after merge) before tagging a release.
- **Publish:** `pypi` environment reviewers on tag push only.

Future optimization: `sccache` with an org S3 bucket when `SCCACHE_BUCKET` and IAM OIDC are available (not configured today).

## PyPI trusted publisher

Configure **after** [`.github/workflows/publish-python-sdk.yml`](../../../.github/workflows/publish-python-sdk.yml) is on `master`.

1. Register the project on [pypi.org](https://pypi.org/) as **`arcflow`** (if not already created).
2. Open **Publishing** → **Add a new pending publisher** and enter:

| Field | Value |
|-------|--------|
| PyPI project name | `arcflow` |
| Owner | `isonlycoolie` |
| Repository name | `ArcFlow` |
| Workflow name | `publish-python-sdk.yml` |
| Environment name | `pypi` |

3. Save. PyPI validates the workflow file path on the default branch.

Publishing uses **OIDC** (no long-lived API token in the repository). Only the `publish` job in `publish-python-sdk.yml` requests the `pypi` environment.

## GitHub environment `pypi`

In the **ArcFlow** repository on GitHub:

1. **Settings** → **Environments** → **New environment** → name: `pypi`.
2. **Deployment branches:** limit to tags matching `sdk-python/v*` (recommended).
3. **Required reviewers:** add at least one maintainer for production uploads.
4. Do not allow bypass for users who should not publish to PyPI.

## Release flow

1. **Bump version** in [`sdk-python/pyproject.toml`](../../../sdk-python/pyproject.toml) only (semver). Open a PR; ensure [SDK Python](https://github.com/isonlycoolie/ArcFlow/actions/workflows/sdk-python.yml) (fast) is green on the PR.
2. **Merge to `master`.** Wait for [SDK Python Compat](https://github.com/isonlycoolie/ArcFlow/actions/workflows/sdk-python-compat.yml) to pass on `master` before tagging.
3. **Create and push an annotated tag** on that commit:

   ```bash
   git tag -a sdk-python/v0.3.0 -m "Python SDK 0.3.0"
   git push origin sdk-python/v0.3.0
   ```

   The tag suffix must match `version` in `pyproject.toml` (e.g. tag `sdk-python/v0.3.0` → `version = "0.3.0"`).

4. **Approve** the `pypi` environment deployment when GitHub prompts (if reviewers are configured).
5. **Verify** the [Publish Python SDK](https://github.com/isonlycoolie/ArcFlow/actions/workflows/publish-python-sdk.yml) workflow: validate → test → cibuildwheel (Linux, macOS, Windows) → PyPI sanity → upload → GitHub Release.
6. **Smoke install:**

   ```bash
   pip install "arcflow==0.3.0"
   python -c "from arcflow import Agent, Workflow; print('ok')"
   ```

## Dry run (no upload)

Use **Actions** → **Publish Python SDK** → **Run workflow**:

- **tag:** `sdk-python/v0.3.0` (must match current `pyproject.toml` version)
- **dry_run:** `true` (default)

This runs validate, test, and wheel builds only. The workflow sets `dry_run: true` so **publish** and **github-release** jobs are skipped. PyPI upload and GitHub Release run **only** on tag push, not on `workflow_dispatch`.

## Local checks

From `sdk-python/`:

```bash
make verify-tag TAG=sdk-python/v0.3.0
make check-pypi VERSION=0.3.0
make lint
make test-publish-gate
make build-wheels   # requires Docker on Linux for manylinux wheels
```

## Failure handling

| Failure | Action |
|---------|--------|
| Tag ≠ `pyproject.toml` version | Fix tag or version PR; never retag over a bad release |
| Version already on PyPI | Bump semver in `pyproject.toml`; new tag |
| OIDC / 403 on publish | Re-check trusted publisher fields and `pypi` environment name |
| cibuildwheel fails | See workflow logs; confirm full monorepo checkout and Rust toolchain |

## First release checklist

1. Merge all PyPI pipeline PRs to `master`.
2. Configure PyPI trusted publisher (table above).
3. Create GitHub environment `pypi`.
4. Run workflow_dispatch with `dry_run: true` for `sdk-python/v0.3.0`.
5. Push tag `sdk-python/v0.3.0` and approve environment.
6. Confirm [PyPI project](https://pypi.org/project/arcflow/) and GitHub Release assets.
