# Security checklist for maintainers

Steps to run before opening the repository as public/open source. The full release checklist (metadata, GitHub settings, community templates) is in [`OPEN_SOURCE.md`](../OPEN_SOURCE.md).

1. Run a secret scan on the repository history and current tree (detect-secrets, truffleHog)
2. If any secrets are found, rotate the secrets and scrub history using `git filter-repo` or BFG
3. Replace any default credentials in compose/files with placeholders (no production defaults in repo) — **done** in `docker/docker-compose.*.yml`
4. Ensure `.gitignore` includes local secret files (e.g., `.env`, `scripts/.pypi-token`) — **done**
5. Add `SECURITY.md` and `CODE_OF_CONDUCT.md` — **done**
6. `cargo audit` runs in **CI Full** ([`.github/workflows/ci-full.yml`](../.github/workflows/ci-full.yml)); enable Dependabot/npm audit in GitHub after going public
7. Enable GitHub secret scanning and Dependabot alerts in the repository settings (maintainer, post-publish)
8. Verify CI logs do not echo environment variables or secrets; fix any unsafe steps
9. Verify license, contributor guide, and governance docs are present — see [`OPEN_SOURCE.md`](../OPEN_SOURCE.md) table
10. Run `bash scripts/ci-smoke.sh` locally (includes secret-pattern grep and commit-size on HEAD)

After publishing:

- Monitor Dependabot and vulnerability alerts
- Triage security reports quickly and follow the disclosure procedure
