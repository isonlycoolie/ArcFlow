# ArcFlow Operator Dashboard (specification)

This folder holds **design and technical specifications** for the ArcFlow operator dashboard. It does not contain dashboard application code.

**Implementation and spec copy:** [ArcFlow-Dashboard](https://github.com/isonlycoolie/ArcFlow-Dashboard) (private) — Vite UI, `spec/`, and dashboard CI. Submodule in the [meta-repo template](../deploy/meta-repo-template/).

Keep OSS `dashboard/spec/` in sync when changing admin API bindings; mirror updates to the private repo.

## Audience

- Dashboard frontend engineers
- Product designers
- Operators integrating Sites, Knowledge, and Chat publish flows

## Spec index

| Document | Purpose |
|----------|---------|
| [spec/01-product-vision.md](spec/01-product-vision.md) | Personas, tiers, non-goals |
| [spec/02-information-architecture.md](spec/02-information-architecture.md) | Navigation and screen map |
| [spec/03-admin-api-contract.md](spec/03-admin-api-contract.md) | Admin REST routes, request/response JSON |
| [spec/04-api-enforcement.md](spec/04-api-enforcement.md) | Auth headers, scoped keys, 403 matrix |
| [spec/05-security-model.md](spec/05-security-model.md) | Tokens, origins, SEC-1, browser boundaries |
| [spec/06-features-tier1.md](spec/06-features-tier1.md) | R1 features (Sites, Knowledge, Chat) |
| [spec/07-features-tier2.md](spec/07-features-tier2.md) | Future tier (usage, BYO keys, builder) |
| [spec/08-ui-states-and-errors.md](spec/08-ui-states-and-errors.md) | Loading, empty, error, one-time token UX |
| [spec/09-exit-criteria.md](spec/09-exit-criteria.md) | Definition of done for dashboard v1 |

## Related OSS docs

- [Meta-repo deployment guide](../documentation/deployment/overview.md#meta-repo-layout)
- [Static site examples](../examples/static/README.md)
- [Static product overview](../documentation/static-product/overview.md)

## Status

**Dashboard UI** is deferred. This spec folder is the production-readiness handoff artifact until the dashboard repo ships v1.
