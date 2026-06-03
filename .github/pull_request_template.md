## Summary

<!-- What changed and why (1–3 sentences) -->

## Target branch

- [ ] **`development`**, feature / fix (fast CI required)
- [ ] **`master`**, release promotion from `development` only (CI Full required on head SHA)

> <p style="color:red"><strong>One engine, every surface.</strong> Orchestration lives in arcflow-core (Rust). SDKs serialize workflow definitions into the Runtime Contract Specification (RCS), invoke the engine, and deserialize results. A fix in retry policy or recovery ships once and applies everywhere. <br><strong>Warning:</strong> keep commits focused, a single commit must not exceed 200 insertions, split unrelated changes into separate branches, and do not commit directly to `master` or `development`.</p>

## Checklist

- [ ] `bash scripts/ci-local.sh` passed locally
- [ ] Commits are ≤100 code lines each (`bash scripts/check-commit-size.sh`)
- [ ] `bash scripts/install-git-hooks.sh` run on this clone (pre-push + commit-msg hooks)

### For PRs to `master` only

- [ ] Head branch is **`development`**
- [ ] **CI Full** succeeded on this PR's head commit (daily 06:00 UTC or manual dispatch)
- [ ] **Release promotion gate** is green

### For documentation changes

- [ ] `node scripts/verify-documentation-prose.mjs` passed
- [ ] WebApp docs export / `docs:lint` updated if publishing to arcflows.vercel.com
