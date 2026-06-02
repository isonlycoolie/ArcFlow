
# Dashboard specification

The ArcFlow operator dashboard is specified in OSS and implemented in a **private repository**. The UI is **deferred under Operator dashboard UI** until exit criteria pass in [ArcFlow-Dashboard](https://github.com/isonlycoolie/ArcFlow-Dashboard.git) CI. Operators today use the admin API, OSS shell scripts, or the v0 starter scaffold.

## Repository split

| Repository | Visibility | Contents |
|------------|------------|----------|
| [ArcFlow](https://github.com/isonlycoolie/ArcFlow) (OSS) | Public | Operator specs on this site, `deploy/arcflow-dashboard-v0/` starter |
| [ArcFlow-Dashboard](https://github.com/isonlycoolie/ArcFlow-Dashboard.git) | Private | Operator UI, dashboard CI, operator `.env` |

Dashboard implementation MUST NOT change admin API semantics without updating OSS spec first.

## OSS specification documents

Published operator docs on this site:

| Document | Content |
|----------|---------|
| [Dashboard spec](dashboard-spec.md) | Personas, tiers, non-goals |
| [Dashboard spec](dashboard-spec.md) | Navigation and screen map |
| [Admin API reference](../operator/admin-api-reference.md) | Admin REST routes (matches §13) |
| [API key management](../security/api-key-management.md) | Auth headers, 403 matrix |
| [Security model](../static-product/security-model.md) | Tokens, origins, trace data policy |
| [Dashboard spec](dashboard-spec.md) | R1 feature scope |
| [Dashboard spec](dashboard-spec.md) | Future tier features |
| [Dashboard spec](dashboard-spec.md) | UI states and error mapping |
| [Dashboard spec](dashboard-spec.md) | Definition of done (private CI) |

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
3. Use the [Admin API reference](admin-api-reference.md) (submodule path in meta-repo).
4. Dev: `npm run dev` on port **5174**; requires `arcflow-server` with matching `ARCFLOW_ADMIN_API_KEY`.

Admin key must stay in BFF env, not Vite client bundle. See [API key management](../security/api-key-management.md).

## Meta-repo consumption

Private platform repos submodule both projects. Template: `deploy/meta-repo-template/`.

```text
ArcFlow-Platform/
 arcflow/ → OSS submodule
 dashboard/ → ArcFlow-Dashboard submodule
 docker-compose.yml
```

Convention ports: server 8080, relay 8090, dashboard dev 5174.

Guide: [Deployment overview — Meta-repo layout](../deployment/overview.md#meta-repo-layout).

## Operator dashboard UI status

| Item | Status |
|------|--------|
| OSS operator specification on this site | Complete |
| OSS static scripts (`scripts/static-*.sh`) | Done |
| Meta-repo template | Done |
| Dashboard UI in private repo | **Deferred** until E/S/D checklists in [Dashboard spec](dashboard-spec.md) pass |

Do not document dashboard UI features as shipped in OSS releases.

## Building operator tooling today

Without the dashboard UI:

| Approach | Use when |
|----------|----------|
| [Admin API reference](admin-api-reference.md) + curl | Automation, CI |
| `scripts/static-provision-site.sh` etc. | Parity testing |
| Custom BFF + internal UI | Enterprise operator portal |
| `deploy/arcflow-dashboard-v0/` | Starting point for private repo |

Admin routes are stable; bind new tools to the [Admin API reference](admin-api-reference.md).

## Normative contracts

Dashboard and server integrations should cross-check:

- [HTTP API reference](../server/http-api-reference.md) (note: partially stale, K-10)
- [workflow schema](../contracts/rcs-schema.md)
- [Trace events (normative)](../contracts/trace-events-normative.md)

## Related pages

- [Sites management](sites-management.md)
- [Deployment overview](../deployment/overview.md)
- [Trace data policy compliance](../security/sec-1-compliance.md)
