# Open source release checklist

ArcFlow is structured for public release: dual license, contributor guide, security policy, CI secret scans, and normative contracts. Use this checklist when flipping the GitHub repository from private to **public**.

## Before making the repository public

### 1. Secrets and history

- [ ] Run a history scan: `detect-secrets scan` or [TruffleHog](https://github.com/trufflesecurity/trufflehog) on the full clone
- [ ] If anything real was committed, **rotate** the credential and rewrite history (`git filter-repo` / BFG), then force-push only after team agreement
- [ ] Confirm CI secret-pattern job passes: `bash scripts/ci-smoke.sh` (includes grep for `sk-…`, `AKIA…`, PEM private keys)
- [ ] Confirm local artifacts stay untracked: `.env`, `scripts/.pypi-token`, `.secrets.baseline`, `trufflehog.json`

### 2. Repository metadata (GitHub Settings)

- [ ] Set description, website (`https://arcflow.dev` when live), and topics: `rust`, `ai-agents`, `workflow`, `llm`, `self-hosted`
- [ ] Enable **Issues** and **Discussions** (optional)
- [ ] Enable **Security → Private vulnerability reporting** and **Dependabot alerts**
- [ ] Enable **Secret scanning** (GitHub Advanced Security if available)
- [ ] Add license in GitHub UI: **MIT** (matches `LICENSE-MIT`; Apache-2.0 is also offered in-tree)
- [ ] Default branch: `development` (see [`.github/BRANCH_POLICY.md`](.github/BRANCH_POLICY.md))

### 3. Files in this repo (should already be present)

| Artifact | Path |
|----------|------|
| License (dual) | [`LICENSE`](LICENSE), [`LICENSE-MIT`](LICENSE-MIT), [`LICENSE-APACHE-2.0`](LICENSE-APACHE-2.0) |
| Contributing | [`CONTRIBUTING.md`](CONTRIBUTING.md) |
| Code of conduct | [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md) |
| Security policy | [`SECURITY.md`](SECURITY.md) |
| Maintainer security steps | [`security/SECURITY_CHECKLIST.md`](security/SECURITY_CHECKLIST.md) |
| Env template | [`.env.example`](.env.example) |
| Trademark | [`TRADEMARK.md`](TRADEMARK.md) |

### 4. Local verification

```bash
bash scripts/ci-smoke.sh          # fast gate (~1 min, no Rust compile)
bash scripts/install-git-hooks.sh
```

On Windows (PowerShell): `.\scripts\ci-local.ps1 -Smoke`

For a release-candidate branch before promoting `development` → `master`:

```bash
bash scripts/ci-local-full.sh
```

### 5. What stays outside this repository

These are intentional and documented in [`CONTRIBUTING.md`](CONTRIBUTING.md):

- `webapp/` — docs site export ([ArcFlow-WebApp](https://github.com/isonlycoolie/ArcFlow-WebApp))
- Operator dashboard UI — [ArcFlow-Dashboard](https://github.com/isonlycoolie/ArcFlow-Dashboard) (OSS spec + `deploy/arcflow-dashboard-v0/` starter here)
- `docs/` — internal ADR pipeline (gitignored)
- Sprint plans, `.cursor/`, local tokens

## After going public

- [ ] Watch **CI** on `development` and scheduled **CI Full**
- [ ] Triage **Dependabot** and security advisories per [`SECURITY.md`](SECURITY.md)
- [ ] Publish Python SDK only from tagged releases (`sdk-python/v*`) via existing workflow
- [ ] Update any hardcoded clone URLs if the org/repo moves (search for `isonlycoolie/ArcFlow`)

## Community

- Bugs and features: GitHub Issues (templates under `.github/ISSUE_TEMPLATE/`)
- Security: **do not** open public issues with exploit details — use [Security Advisories](https://github.com/isonlycoolie/ArcFlow/security/advisories)
- Conduct: see [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md)
