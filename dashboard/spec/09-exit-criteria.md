# Exit criteria

Dashboard v1 is complete when an operator can onboard a static site **without shell scripts**, matching OSS script outcomes.

## Functional checklist

| # | Criterion | Verification |
|---|-----------|--------------|
| E1 | Create site and obtain relay URL + token | Manual or E2E against admin API |
| E2 | Ingest knowledge text | `chunks_ingested > 0` |
| E3 | Publish chat workflow with instructions | Response includes name + version |
| E4 | Update allowed origins | PATCH succeeds; Relay rejects wrong Origin |
| E5 | Rotate site token | Old token fails; new token works |
| E6 | Copy deploy env snippet | Static example app connects |

## Parity with OSS scripts

Dashboard flows must produce the same server state as:

- `scripts/static-provision-site.sh`
- `scripts/static-ingest-knowledge.sh`
- `scripts/static-publish-chat.sh`
- `scripts/static-smoke.sh` (end-to-end)

## Security checklist

| # | Criterion |
|---|-----------|
| S1 | No admin API key in client bundle (BFF or dev-only exception documented) |
| S2 | Site token shown once on create and rotate |
| S3 | SEC-1 CI passes on dashboard repo |
| S4 | No secrets in browser localStorage except operator session cookie |

## Documentation checklist

| # | Criterion |
|---|-----------|
| D1 | Meta-repo README links to dashboard submodule |
| D2 | Operator quickstart references dashboard or scripts equivalently |
| D3 | This `dashboard/spec/` folder remains source of truth for API bindings |

## Production readiness alignment

| Plan item | Status with spec-only handoff |
|-----------|-------------------------------|
| FP-3.01 Dashboard UI | **Deferred** — spec complete |
| FP-3.02 Static scripts | Done (OSS) |
| FP-3.03 Meta-repo template | Done |
| FP-3.04 Relay hardening | Separate workstream |
| FP-3.06 E2E without dashboard | Python E2E against admin API |

## Sign-off owners

| Role | Responsibility |
|------|----------------|
| Product | Tier 1 scope matches persona goals |
| Security | BFF and token UX reviewed |
| Platform | Admin API contract matches `arcflow-server` handlers |

When all E*, S*, and D* items pass in the dashboard repository CI, FP-3.01 may be marked complete in the production readiness plan.
