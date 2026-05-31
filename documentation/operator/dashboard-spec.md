**Audience:** `[operator]` `[platform]`

# Dashboard specification

The ArcFlow operator dashboard is specified in OSS and implemented in a **private repository**. The UI is **deferred under FP-3.01** until exit criteria pass in [ArcFlow-Dashboard](https://github.com/isonlycoolie/ArcFlow-Dashboard.git) CI. Operators today use the admin API, OSS shell scripts, or the v0 starter scaffold.

## Repository split

| Repository | Visibility | Contents |
|------------|------------|----------|
| [ArcFlow](https://github.com/isonlycoolie/ArcFlow) (OSS) | Public | `dashboard/spec/*`, `deploy/arcflow-dashboard-v0/` starter |
| [ArcFlow-Dashboard](https://github.com/isonlycoolie/ArcFlow-Dashboard.git) | Private | Operator UI, dashboard CI, operator `.env` |

Dashboard implementation MUST NOT change admin API semantics without updating OSS spec first.

## OSS specification documents

Authoritative spec folder: `dashboard/spec/`.

| Document | Content |
|----------|---------|
| [01-product-vision.md](../../dashboard/spec/01-product-vision.md) | Personas, tiers, non-goals |
| [02-information-architecture.md](../../dashboard/spec/02-information-architecture.md) | Navigation and screen map |
| [03-admin-api-contract.md](../../dashboard/spec/03-admin-api-contract.md) | Admin REST routes (matches §13) |
| [04-api-enforcement.md](../../dashboard/spec/04-api-enforcement.md) | Auth headers, 403 matrix |
| [05-security-model.md](../../dashboard/spec/05-security-model.md) | Tokens, origins, SEC-1 |
| [06-features-tier1.md](../../dashboard/spec/06-features-tier1.md) | R1 feature scope |
| [07-features-tier2.md](../../dashboard/spec/07-features-tier2.md) | Future tier features |
| [08-ui-states-and-errors.md](../../dashboard/spec/08-ui-states-and-errors.md) | UI states and error mapping |
| [09-exit-criteria.md](../../dashboard/spec/09-exit-criteria.md) | Definition of done (private CI) |

Documentation mirror: [Admin API reference](admin-api-reference.md), [Sites management](sites-management.md).

## Product scope (Tier 1)

The dashboard replaces shell scripts for static site operators:

1. Create Relay sites and show one-time credentials (`VITE_*` env vars).
2. Ingest knowledge text into site vector namespace.
3. Set chat instructions and publish default `chat` workflow version.
4. Edit allowed origins, rate limits, rotate site tokens.

Non-goals for v1: ArcFlow Cloud billing, visual workflow editor, end-user OAuth to dashboard, Stripe UI.

## Private repo bootstrap

1. Clone [ArcFlow-Dashboard](https://github.com/isonlycoolie/ArcFlow-Dashboard.git).
2. Sync from OSS starter: `deploy/arcflow-dashboard-v0/`.
3. Point API contract at `dashboard/spec/03-admin-api-contract.md` (submodule path in meta-repo).
4. Dev: `npm run dev` on port **5174**; requires `arcflow-server` with matching `ARCFLOW_ADMIN_API_KEY`.

Admin key must stay in BFF env, not Vite client bundle. See [04-api-enforcement.md](../../dashboard/spec/04-api-enforcement.md).

## Meta-repo consumption

Private platform repos submodule both projects. Template: `deploy/meta-repo-template/`.

```text
ArcFlow-Platform/
  arcflow/      → OSS submodule
  dashboard/    → ArcFlow-Dashboard submodule
  docker-compose.yml
```

Convention ports: server 8080, relay 8090, dashboard dev 5174.

Guide: [contracts/guides/deployment/meta-repo.md](../../contracts/guides/deployment/meta-repo.md).

## FP-3.01 status

| Item | Status |
|------|--------|
| OSS `dashboard/spec/` handoff | Complete |
| OSS static scripts (`scripts/static-*.sh`) | Done |
| Meta-repo template | Done |
| Dashboard UI in private repo | **Deferred** until E/S/D checklists in [09-exit-criteria.md](../../dashboard/spec/09-exit-criteria.md) pass |

Do not document dashboard UI features as shipped in OSS releases.

## Building operator tooling today

Without the dashboard UI:

| Approach | Use when |
|----------|----------|
| [Admin API reference](admin-api-reference.md) + curl | Automation, CI |
| `scripts/static-provision-site.sh` etc. | Parity testing |
| Custom BFF + internal UI | Enterprise operator portal |
| `deploy/arcflow-dashboard-v0/` | Starting point for private repo |

Admin routes are stable; bind new tools to `03-admin-api-contract.md`.

## Normative contracts

Dashboard and server integrations should cross-check:

- [contracts/normative/runtime/server-api-v1.md](../../contracts/normative/runtime/server-api-v1.md) (note: partially stale, K-10)
- [contracts/normative/rcs/v1.schema.json](../../contracts/normative/rcs/v1.schema.json)
- [contracts/normative/observability/trace-events-v1.md](../../contracts/normative/observability/trace-events-v1.md)

## Related pages

- [Sites management](sites-management.md)
- [Deployment overview](../deployment/overview.md)
- [SEC-1 compliance](../security/sec-1-compliance.md)

## Source

Derived from [ARCFLOW-FULL-CAPABILITIES-REFERENCE.md](../../docs/_draft/ARCFLOW-FULL-CAPABILITIES-REFERENCE.md) §21; `dashboard/spec/` index; FP-3.01 / §27 known gaps.
