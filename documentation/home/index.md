
# ArcFlow documentation

ArcFlow is a self-hosted AI workflow runtime written in Rust. One engine (`arcflow-core`) powers Python and TypeScript SDKs, the HTTP server, Relay, the static browser SDK, CLI, VS Code extension, and an alpha WASM build. Workflows are defined as RCS v1 JSON: agents, steps, optional graph routing, tools, vector memory, human-in-the-loop gates, and external callbacks.

This site explains what the system does, which surface to use, and how to run workflows safely in production. Normative wire formats are in [The RCS contract](../concepts/the-rcs-contract.md) and [Trace events (normative)](../contracts/trace-events-normative.md).

## Four entry paths

Pick the path that matches your job today. Each path links into deeper guides without assuming you have read everything else.

**Build workflows in code.** Start with the [Getting started curriculum](../getting-started/README.md): [How ArcFlow thinks](../getting-started/fundamentals/01-how-arcflow-thinks.md), then [Linear pipelines](../getting-started/writing-workflows/01-linear-pipelines.md). For the fastest run only, use [First workflow in five minutes](../getting-started/first-workflow-in-five-minutes.md). Read [What is ArcFlow?](../concepts/what-is-arcflow.md) for personas and non-goals, then [The RCS contract](../concepts/the-rcs-contract.md) and [Execution model](../concepts/execution-model.md) when you need graph routing, recovery, or HITL.

**Integrate via HTTP.** Start with [Server API quickstart](../getting-started/quickstart-server-api.md) for `POST /v1/runs`, idempotency, and polling. Pair that with [Architecture overview](../concepts/architecture-overview.md) for the backend integration flow and [Surfaces and when to use them](../concepts/surfaces-and-when-to-use-them.md) to confirm Postgres is required for server runs.

**Operate the static product.** Start with [Static site chatbot lesson](../getting-started/paths/static-site-chatbot.md) for a guided path, or [Static product overview](../static-product/overview.md) for reference depth. The browser path runs through Relay so LLM keys never ship in frontend bundles. [SEC-1 and data safety](../concepts/sec-1-and-data-safety.md) covers what traces may contain.

**Review security and compliance.** Start with [SEC-1 and data safety](../concepts/sec-1-and-data-safety.md), then [Maturity and known gaps](../concepts/maturity-and-known-gaps.md) for deferred items (server SSE, dashboard UI, CLI validate, graph recovery resume, OTel metrics). Admin auth and webhook HMAC details live under [security/](../security/sec-1-compliance.md).

## Which surface should I use?

All execution paths converge on `arcflow-core::WorkflowEngine`. Behavior is consistent across surfaces. Differences are auth boundaries, persistence, and I/O.

| Surface | Package / binary | Primary use | Postgres required |
|---------|------------------|-------------|-------------------|
| Python SDK | `sdk-python` | Scripts, notebooks, backend services | Only when recovery or registry is enabled |
| TypeScript SDK | `sdk-typescript` | Node services, VS Code, tests | Same as Python |
| arcflow-server | `server/arcflow-server` | HTTP API, registry, admin, persistence | **Yes** for `POST /v1/runs` |
| arcflow-relay | `server/arcflow-relay` | Browser proxy, origin check, rate limit | No (stateless proxy) |
| arcflow-static | `packages/arcflow-static` | Browser `runPublished()` | No (calls Relay or server) |
| arcflow CLI | `cli/arcflow-cli` | Local run, trace, migrate | For `migrate` only |
| VS Code extension | `extensions/vscode-arcflow` | Authoring, debug, graph view | No |
| WASM | `runtime/arcflow-wasm` | Edge Workers (alpha) | No |

| Scenario | Recommended surface |
|----------|---------------------|
| Internal batch job | Python or TypeScript SDK embedded |
| Multi-tenant SaaS backend | arcflow-server plus your auth layer |
| Public website chat widget | Relay + static SDK + admin publish |
| Local debugging | SDK or CLI, optional VS Code |
| Operator runbooks | CLI, SQL, admin API; dashboard UI deferred (FP-3.01) |

For a full comparison and Postgres notes, see [Surfaces and when to use them](../concepts/surfaces-and-when-to-use-them.md).

## Concepts worth reading early

| Topic | Page |
|-------|------|
| Personas and non-goals | [What is ArcFlow?](../concepts/what-is-arcflow.md) |
| Layers, static product flow, backend flow | [Architecture overview](../concepts/architecture-overview.md) |
| RCS JSON, contract-first design | [The RCS contract](../concepts/the-rcs-contract.md) |
| Linear vs graph, run states | [Execution model](../concepts/execution-model.md) |
| Trace metadata rules | [SEC-1 and data safety](../concepts/sec-1-and-data-safety.md) |
| Production vs deferred features | [Maturity and known gaps](../concepts/maturity-and-known-gaps.md) |

## Site map

| Section | Purpose |
|---------|---------|
| [concepts/](../concepts/what-is-arcflow.md) | Architecture, RCS, execution, maturity |
| [getting-started/](../getting-started/README.md) | Curriculum: install, lessons, outcome paths |
| [guides/](../guides/workflows/linear-workflows.md) | Task-oriented how-to guides |
| [sdks/](../sdks/python/overview.md) | Python and TypeScript SDK reference |
| [server/](../server/overview.md) | HTTP server and persistence |
| [relay/](../relay/overview.md) | Browser-facing proxy |
| [static-product/](../static-product/overview.md) | Sites, knowledge, published workflows |
| [deployment/](../deployment/overview.md) | Docker, production, migrations |
| [security/](../security/sec-1-compliance.md) | Auth, webhooks, self-hosted hardening |
| [tutorials/](../tutorials/track-a-first-workflow.md) | Tracks A through H |
