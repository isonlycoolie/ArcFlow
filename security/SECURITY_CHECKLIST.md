# Security checklist for maintainers

Steps to run before opening the repository as public/open source:

1. Run a secret scan on the repository history and current tree (detect-secrets, truffleHog)
2. If any secrets are found, rotate the secrets and scrub history using `git filter-repo` or BFG
3. Replace any default credentials in compose/files with placeholders (no production defaults in repo)
4. Ensure `.gitignore` includes local secret files (e.g., `.env`, `scripts/.pypi-token`)
5. Add `SECURITY.md` and `CODE_OF_CONDUCT.md` (done)
6. Add CI jobs to run `cargo audit`, `pip-audit`/`safety`, and `npm audit`
7. Enable GitHub secret scanning and Dependabot alerts in the repository settings
8. Verify CI logs do not echo environment variables or secrets; fix any unsafe steps
9. Verify license, contributor guide, and governance docs are present
10. Run `bash scripts/check-commit-size.sh` and CI smoke locally

After publishing:

- Monitor Dependabot and vulnerability alerts
- Triage security reports quickly and follow the disclosure procedure
