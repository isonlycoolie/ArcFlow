**Audience:** `[developer]` `[platform]` `[operator]` `[compliance]`

# Maturity and known gaps

This page states what ArcFlow ships today at production quality, what is alpha or partial, and what is explicitly deferred. Do not document deferred items as available in tutorials or sales-facing copy. When a feature is partial, describe the gap honestly so operators can plan workarounds.

Verified baseline: 2026-05-31 against `arcflow-core`, `arcflow-server`, and migrations `20240531000001` through `000007`.

## Maturity matrix

| Area | Maturity | Notes |
|------|----------|-------|
| Linear workflows | **Production** | SDK and server paths |
| Graph execution | **Production** | Routing, joins, parallel fan-out; graph **recovery resume partial** |
| RAG / vector memory | **Production** | Requires Qdrant and non-stub embedding provider in prod |
| Human-in-the-loop (HITL) | **Production** | Requires `recovery_enabled` and Postgres |
| External callbacks | **Production** | HMAC verification on webhook ingress |
| Static product (Relay + admin + static SDK) | **Production** | Sites, ingest, semver publish |
| Server SSE streaming | **Deferred** | FP-2: `GET /v1/runs/{id}/events` not implemented |
| OpenTelemetry metrics | **Alpha** | FP-4: opt-in export stabilizing |
| CLI `arcflow validate` | **Stub** | FP-5.04: use JSON Schema in CI until shipped |
| Operator dashboard UI | **Deferred** | FP-3.01: private ArcFlow-Dashboard repo; OSS spec complete |
| Edge WASM | **Alpha** | Linear stub only; not for production |

### Production areas in brief

**Linear workflows** are the default path for SDK and `POST /v1/runs`. Recovery resume from last committed step is supported when persistence is on.

**Graph execution** handles conditional edges, branch termination (`to: null`), join nodes, and checkpoints written to Postgres. Execution of new runs is production-ready; **resuming a failed graph from checkpoint is not** (see FP-1.01 below).

**RAG** uses Qdrant collections per namespace, hybrid dense/sparse when configured, optional Cohere rerank. Traces record retrieval metrics only (SEC-1).

**HITL** interrupts with `Interrupted` status, approve/reject via server API, timeout and duplicate-approve error codes documented in RCS.

**Static product** covers admin site lifecycle, knowledge ingest, chat workflow publish, Relay origin and rate limits, and `runPublished()` in the browser.

## Known gaps and deferred work

| ID | Gap | Status | Workaround / plan |
|----|-----|--------|-------------------|
| **FP-1.01** | Graph recovery resume from checkpoint | **Partial** | Schema and `persist_graph_checkpoint` exist; resume dispatch incomplete. Plan: `feat/fp-1-graph-recovery`. Do not rely on mid-graph resume in production. Linear recovery works. |
| **FP-2** | Server SSE `/v1/runs/{id}/events` | **Deferred** | Poll `GET /v1/runs/{id}` or trace. SDK `run_stream()` works in-process. Browser polls Relay trace for `TokenEmitted`. Plan: `feat/fp-2-server-streaming`. |
| **FP-3.01** | Operator dashboard UI | **Deferred** | OSS [dashboard/spec/](../../dashboard/spec/) and admin API are source of truth. UI lives in private [ArcFlow-Dashboard](https://github.com/isonlycoolie/ArcFlow-Dashboard.git). Use admin API, `scripts/static-*.sh`, or v0 starter until [09-exit-criteria.md](../../dashboard/spec/09-exit-criteria.md) passes in private CI. |
| **FP-4** | Stable OTel metrics export | **Alpha** | Core runtime does not need OTel. Enable only with review of label cardinality and SEC-1. Plan: `feat/fp-4-observability`. |
| **FP-5.04** | Full `arcflow validate` schema check | **Stub** | CLI command exists but does not fully validate. Validate against [v1.schema.json](../../contracts/normative/rcs/v1.schema.json) in CI. Plan: `feat/fp-5-cli-validate`. |
| FP-7 | Production signoff checklist | Tracking | See FINAL-PRODUCTION-READINESS-PLAN in repo planning docs |

### FP-1.01: Graph recovery (partial)

When `recovery_enabled` is true, graph runs persist `current_node_id`, `graph_iteration_count`, and `pending_join` in `arcflow_recovery_state`. That supports observability and future resume. **Dispatch to continue from those fields after a crash is incomplete.** Treat graph runs as non-resumable for SLA planning until FP-1.01 closes.

### FP-2: Server streaming (deferred)

Do not document SSE on the server as shipped. Client integrations should use polling or SDK streaming inside your backend, not expect browser SSE from `arcflow-server`.

### FP-3.01: Dashboard UI (deferred)

Operators today:

- Call admin routes (`POST /v1/admin/sites`, knowledge ingest, chat publish) with `ARCFLOW_ADMIN_API_KEY`
- Run `arcflow migrate up` and health checks via CLI
- Query Postgres for runs and traces
- Sync private dashboard repo from [deploy/arcflow-dashboard-v0/](../../deploy/arcflow-dashboard-v0/) when building UI

OSS spec must lead API changes; dashboard implementation must not drift admin semantics without spec updates.

### FP-4: OTel (alpha)

`ARCFLOW_OTEL_ENABLED` and `ARCFLOW_OTLP_ENDPOINT` exist for early adopters. Metrics export may change. Not required for workflow correctness.

### FP-5.04: CLI validate (stub)

`arcflow validate` is not a substitute for schema validation in CI. Use the normative JSON Schema until the CLI implements full checks.

## Feature parity snapshot

Not every surface exposes every capability. Highlights:

| Capability | Server | Relay | Static SDK | CLI | WASM |
|------------|:------:|:-----:|:----------:|:---:|:----:|
| Graph workflows | Y | Y | via publish | n/a | N |
| Server SSE | N (FP-2) | n/a | poll | n/a | N |
| Recovery | Y | n/a | n/a | migrate | N |
| Admin / sites | Y | n/a | n/a | n/a | N |

Full matrix: capabilities reference Appendix I.

## Contract drift warning

[server-api-v1.md](../../contracts/normative/runtime/server-api-v1.md) may list only legacy routes. Implemented server routes match `server/arcflow-server/src/lib.rs` and capabilities reference Appendix B. Update normative docs when promoting from draft.

## Related pages

- [Surfaces and when to use them](surfaces-and-when-to-use-them.md) for Postgres and Relay requirements
- [Execution model](execution-model.md) for graph and recovery behavior
