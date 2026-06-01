
# Dashboard specification

The ArcFlow operator dashboard is implemented in the **ArcFlow-WebApp** private repository (`webapp/` submodule). OSS spec archive: `webapp/docs/operator/` in that repo. Operators may also use admin API shell scripts.

## Repository split

| Repository | Visibility | Contents |
|------------|------------|----------|
| [ArcFlow](https://github.com/isonlycoolie/ArcFlow) (OSS) | Public | Operator docs on this site, deploy templates |
| [ArcFlow-WebApp](https://github.com/isonlycoolie/ArcFlow-WebApp.git) | Private | Operator UI (Next.js), operator-api, dashboard CI |

Dashboard implementation MUST NOT change admin API semantics without updating OSS spec first.

## OSS specification documents

Published operator docs on this site:

| Document | Content |
|----------|---------|
| [Dashboard spec](dashboard-spec.md) | Personas, tiers, non-goals |
| [Dashboard spec](dashboard-spec.md) | Navigation and screen map |
| [03-admin-api-contract.md](../operator/admin-api-reference.md) | Admin REST routes (matches §13) |
| [API key management](../security/api-key-management.md) | Auth headers, 403 matrix |
| [05-security-model.md](../static-product/security-model.md) | Tokens, origins, SEC-1 |
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

1. Clone [ArcFlow-WebApp](https://github.com/isonlycoolie/ArcFlow-WebApp.git).
2. Spec archive: `webapp/docs/operator/` in that repo.
3. Use the [Admin API reference](admin-api-reference.md).
4. Dev: `npm run dev` on port **5174**; start `operator-api` on **8091**; requires `arcflow-server` with matching `ARCFLOW_ADMIN_API_KEY`.

Admin key must stay in BFF env, not Vite client bundle. See [API key management](../security/api-key-management.md).

## Meta-repo consumption

Private platform repos submodule both projects. Template: `deploy/meta-repo-template/`.

```text
ArcFlow-Platform/
  arcflow/      → OSS submodule
  webapp/       → ArcFlow-WebApp submodule
  docker-compose.yml
```

Convention ports: server 8080, relay 8090, webapp dev 5174, operator-api 8091.

Guide: [contracts/guides/deployment/meta-repo.md](../../contracts/guides/deployment/meta-repo.md).

## FP-3.01 status

| Item | Status |
|------|--------|
| OSS operator specification on this site | Complete |
| OSS static scripts (`scripts/static-*.sh`) | Done |
| Meta-repo template | Done |
| Dashboard UI in ArcFlow-WebApp | **Shipped** (Tier 1); Tier 2 usage/list API pending |

Do not document Tier 2 dashboard features (usage charts, server site list) as shipped.

## Building operator tooling today

Without the dashboard UI:

| Approach | Use when |
|----------|----------|
| [Admin API reference](admin-api-reference.md) + curl | Automation, CI |
| `scripts/static-provision-site.sh` etc. | Parity testing |
| Custom BFF + internal UI | Enterprise operator portal |
| ArcFlow-WebApp clone | Full operator dashboard + auth |

Admin routes are stable; bind new tools to the [Admin API reference](admin-api-reference.md).

## Normative contracts

Dashboard and server integrations should cross-check:

- [HTTP API reference](../server/http-api-reference.md) (note: partially stale, K-10)
- [RCS schema](../contracts/rcs-schema.md)
- [Trace events (normative)](../contracts/trace-events-normative.md)

## Related pages

- [Sites management](sites-management.md)
- [Deployment overview](../deployment/overview.md)
- [SEC-1 compliance](../security/sec-1-compliance.md)
