
# Surfaces and when to use them

ArcFlow exposes one engine through eight surfaces. They differ in where code runs, what credentials exist, and whether Postgres is required. Picking the wrong surface usually means either missing persistence (server runs without Postgres) or leaking secrets (direct server mode from a public website).

All paths call `arcflow-core::WorkflowEngine`. Workflow semantics, trace event kinds, and error codes stay consistent. What changes is auth, I/O boundaries, and which optional features (registry, recovery, admin) are reachable.

## Runtime surface reference

| Surface | Binary / package | Primary use | Postgres required |
|---------|------------------|-------------|-------------------|
| Python SDK | `sdk-python` | Scripts, notebooks, backend services | Only for recovery or registry when enabled |
| TypeScript SDK | `sdk-typescript` | Node services, VS Code, tests | Same as Python |
| arcflow-server | `server/arcflow-server` | HTTP API, registry, admin, persistence | **Yes** for `POST /v1/runs` |
| arcflow-relay | `server/arcflow-relay` | Browser proxy, origin check, rate limit | No (stateless proxy) |
| arcflow-static | `packages/arcflow-static` | Browser `runPublished()` | No (calls Relay or server) |
| arcflow CLI | `cli/arcflow-cli` | Local run, trace, migrate | For `migrate` only |
| VS Code extension | `extensions/vscode-arcflow` | Authoring, debug, graph view | No |
| WASM | `runtime/arcflow-wasm` | Edge Workers (alpha) | No |

## Scenario guide

| Scenario | Recommended surface | Why |
|----------|---------------------|-----|
| Internal batch job | Python or TypeScript SDK embedded | No HTTP hop; keys stay in your process environment |
| Multi-tenant SaaS backend | arcflow-server + your auth layer | Central registry, idempotency, persisted runs and traces |
| Public website chat widget | Relay + static SDK + admin publish | Site token + origin gate; LLM keys on server only |
| Local debugging | SDK or CLI, optional VS Code | Fast iteration; trace via SDK result or `arcflow trace` |
| Operator runbooks | CLI + SQL + admin API | Dashboard UI deferred (FP-3.01); use admin routes and scripts today |
| Edge experiment | WASM (alpha) | Linear stub only; no graph, RAG, or recovery |

## Surface details

### Python SDK (`sdk-python`)

Native extension over `arcflow-core` via PyO3. Supports linear and graph workflows, tools, vector memory, recovery, streaming (`run_stream()`), and in-memory trace on `RunResult`. Use when the workflow runs inside a Python service or notebook and you do not need the HTTP registry unless you call the server separately.

### TypeScript SDK (`sdk-typescript`)

N-API binding to the same Rust engine. Feature parity with Python for in-process execution (see parity matrix in the this documentation site). Powers the VS Code extension and Node integration tests. Browser bundling is possible but [static-product](../static-product/overview.md) is preferred for production browser use.

### arcflow-server

The production HTTP API: `POST /v1/runs`, run status, trace, HITL approve, external callbacks, workflow registry, admin routes for static product. Requires `ARCFLOW_POSTGRESQL_URL` for runtime routes. Returns 503 when the pool is unavailable. Your SaaS layer should map tenant identity to server API keys or scoped static runtime keys.

### arcflow-relay

Stateless proxy for browsers. Validates site token, checks `Origin` against `allowed_origins`, enforces `rate_limit_rpm`. Proxies run create, status, and trace to upstream server with scoped keys. Does not execute workflows itself. BYO deployment example: `examples/relay/byo-docker/`.

### arcflow-static (`@arcflow/static`)

Browser client for published workflows. Production mode is `relay`. `direct` mode hits the server with an API key (dev only). `bff` mode calls your backend, which holds keys. Core API: `runPublished(name, semverRange, input, options)`.

### arcflow CLI

Local `arcflow run`, `arcflow trace <run_id> [--tui]`, `arcflow migrate up`. **`arcflow validate` is a stub** (FP-5.04): schema validation in CI should use [RCS schema](../contracts/rcs-schema.md) until the CLI command ships.

### VS Code extension

In-repo extension for workflow graph visualization, run from workspace, metadata trace timeline. Not marketplace GA. Complements SDK local runs rather than replacing server deployment.

### WASM (alpha)

`runtime/arcflow-wasm` targets Cloudflare Workers and edge experiments. Linear workflow stub only. Graph, RAG, and recovery are not supported. Not recommended for production.

## Postgres: when it matters

| Feature | Embedded SDK | arcflow-server |
|---------|--------------|----------------|
| One-off local run | Optional | N/A |
| `POST /v1/runs` | N/A | Required |
| Recovery / HITL resume | If `recovery_enabled` and URL set | Supported |
| Workflow registry | Server only | Supported |
| Static product sites | Server admin routes | Supported |
| `arcflow migrate up` | Needs URL | Applies before `/ready` passes |

## Related pages

- [Architecture overview](architecture-overview.md) for static product and backend sequence flows
- [What is ArcFlow?](what-is-arcflow.md) for personas
- [Maturity and known gaps](maturity-and-known-gaps.md) for FP-2 (server SSE), FP-3.01 (dashboard), FP-5.04 (CLI validate)
