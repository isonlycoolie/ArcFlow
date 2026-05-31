# ArcFlow documentation

This folder is the tracked user-facing documentation site for ArcFlow. Normative wire formats and JSON schemas remain in [contracts/normative](../contracts/normative/README.md). ADRs and the internal docs pipeline live in the gitignored `docs/` tree.

## Start here

| If you are… | Start with |
|-------------|------------|
| Building workflows in Python or TypeScript | [Getting started curriculum](getting-started/README.md) |
| Fastest first run only | [First workflow in five minutes](getting-started/first-workflow-in-five-minutes.md) |
| Integrating via HTTP | [Server API quickstart](getting-started/quickstart-server-api.md) |
| Public website chat widget | [Static site chatbot lesson](getting-started/paths/static-site-chatbot.md) |
| Operating sites, knowledge, and publish | [Static product overview](static-product/overview.md) |
| Reviewing security and SEC-1 | [SEC-1 and data safety](concepts/sec-1-and-data-safety.md) |

## Site map

| Section | Purpose |
|---------|---------|
| [home/](home/index.md) | Landing and navigation |
| [concepts/](concepts/what-is-arcflow.md) | Architecture, RCS, execution model, maturity |
| [getting-started/](getting-started/README.md) | Curriculum: learn agents, workflows, tools, memory, RAG, paths |
| [guides/](guides/workflows/linear-workflows.md) | Task-oriented how-to guides |
| [sdks/](sdks/python/overview.md) | Python and TypeScript SDK reference |
| [server/](server/overview.md) | HTTP server and persistence |
| [relay/](relay/overview.md) | Browser-facing proxy |
| [static-product/](static-product/overview.md) | Sites, knowledge, published workflows |
| [cli/](cli/overview.md) | Local CLI |
| [vscode/](vscode/overview.md) | IDE extension |
| [wasm/](wasm/edge-alpha.md) | Edge WASM (alpha) |
| [deployment/](deployment/overview.md) | Docker, production, migrations |
| [operator/](operator/admin-api-reference.md) | Admin API and dashboard spec |
| [security/](security/sec-1-compliance.md) | Auth, webhooks, self-hosted hardening |
| [contracts/](contracts/rcs-schema.md) | Narrative contract guides |
| [examples/](examples/catalog.md) | Example catalog and walkthroughs |
| [tutorials/](tutorials/track-a-first-workflow.md) | Tracks A through H |
| [certification/](certification/overview.md) | Self-assessed competency levels |

## Which surface should I use?

| Scenario | Surface | Postgres |
|----------|---------|----------|
| Internal batch job or notebook | Python or TypeScript SDK (embedded) | Only when recovery or registry is enabled |
| Multi-tenant backend you control | arcflow-server + your auth layer | Required for `POST /v1/runs` |
| Public website chat widget | Relay + static SDK + admin publish | Server side only |
| Local debugging | SDK, CLI, or VS Code | Optional |
| Operator runbooks | CLI, SQL, admin API | Yes for production server |

All surfaces call the same Rust engine (`arcflow-core`). Behavior is consistent; boundaries differ in auth, persistence, and I/O.

## Writing and validation

Pages follow the arcflow-documentation skill: report tone, concrete examples, audience tags, and a Source footer citing capabilities reference sections and code paths. Deferred features (FP-1.01 through FP-5.04) are labeled explicitly in [maturity and known gaps](concepts/maturity-and-known-gaps.md).
