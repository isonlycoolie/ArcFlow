
# SEC-1 and data safety

SEC-1 is ArcFlow's trace data policy: observability outputs describe **what happened** (ids, timings, sizes, codes) without exposing **what was said** (prompts, completions, tool payloads, retrieved text). The rule applies to in-memory SDK traces, HTTP trace endpoints, Relay-proxied traces, persisted `arcflow_trace_events` rows, and VS Code timeline exports.

Compliance and platform teams should treat SEC-1 as a hard boundary, not a logging preference. Violations usually mean a new event field or debug flag was added without review.

## What traces must not contain

Traces and persisted trace events **must not** include:

| Category | Examples of forbidden content |
|----------|-------------------------------|
| LLM content | Prompt text, system instructions as sent, completion text |
| Tool I/O | Tool argument JSON, tool result payloads |
| RAG content | Retrieved chunk text, raw embedding inputs |
| PII | Personal data unless your deployment policy explicitly allows it and you have controls |

This aligns with normative guidance in [contracts/guides/observability/](../../contracts/guides/observability/) and [trace-events-v1.md](../contracts/trace-events-normative.md). Engine implementation: `runtime/arcflow-core/src/tracing/events.rs`.

### Why the boundary exists

Static product browsers poll traces through Relay. Operators export traces from Postgres. Third-party SIEM ingestion is easier when payloads are consistently metadata-only. If prompt text appeared in `GET /v1/runs/{id}/trace`, any site visitor with a valid token could exfiltrate conversation content from network tabs.

Run **results** (`GET /v1/runs/{id}` completed payload) may include final output text appropriate to your product policy. SEC-1 governs **trace events**, not necessarily every API field. HITL interrupt payloads return context metadata sized for approvers without SEC-1 violations (e.g. `summary_bytes`, not full transcripts in trace).

## What traces may contain

Allowed fields are structural and metric:

| Allowed | Typical use |
|---------|-------------|
| ids | `run_id`, `step_id`, `trace_id`, workflow name |
| names / roles | `agent_name`, `agent_role`, `tool_name` |
| sizes | `prompt_size_bytes`, `output_size_bytes`, `input_size_bytes`, `chunk_count`, `total_bytes` |
| tokens | `tokens.input`, `tokens.output`, `tokens.total`, deltas in streaming events |
| durations | `duration_ms`, `latency_ms`, `backoff_ms` |
| error codes | RCS `ErrorCode`, `error_message` where defined as safe summary text |
| schema hashes | `input_schema_hash` on tool events |
| provider metadata | `provider_id`, `model_id`, rate limit `retry_after_seconds` |

Example excerpt (safe):

```json
[
  { "kind": "WorkflowStarted", "run_id": "r1", "workflow_name": "demo", "step_count": 2 },
  { "kind": "ProviderRequestSent", "run_id": "r1", "step_id": "s1", "provider_id": "openai", "model_id": "gpt-4o-mini", "prompt_size_bytes": 512 },
  { "kind": "MemoryRetrieved", "run_id": "r1", "step_id": "s1", "agent_name": "researcher", "chunk_count": 5, "total_bytes": 8192 },
  { "kind": "StepCompleted", "run_id": "r1", "step_id": "s1", "duration_ms": 920, "output_size_bytes": 180 }
]
```

Streaming events follow the same rule: `StreamChunkReceived` records `chunk_bytes`; `TokenEmitted` records token deltas, not raw token strings in trace storage.

## Where traces are exposed

| Surface | Access pattern |
|---------|----------------|
| Python / TypeScript SDK | `result.trace` in memory; export JSON locally |
| CLI | `arcflow trace <run_id> [--tui]` |
| arcflow-server | `GET /v1/runs/{id}/trace` |
| arcflow-relay | `GET /v1/sites/{site_id}/runs/{id}/trace` |
| PostgreSQL | `arcflow_trace_events` when persistence enabled |
| VS Code extension | Timeline from exported trace |

Application logs and webhook handlers are separate surfaces. External callback bodies must not be logged raw. Verify HMAC on `POST /v1/runs/{id}/external/{binding_id}` and keep secrets in `ARCFLOW_WEBHOOK_SECRET`.

## Self-hosted keys

LLM, embedding, and rerank keys belong in environment variables on the server or SDK host (`OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `COHERE_API_KEY`, etc.). Agent definitions reference `api_key_env`, not inline secrets.

Browser bundles must not contain provider keys. Production static sites use Relay with site tokens. `allow_inline: false` on sites blocks inline workflow overrides from the browser.

## OpenTelemetry note

Opt-in metrics via `ARCFLOW_OTEL_ENABLED` and `ARCFLOW_OTLP_ENDPOINT` are **alpha (FP-4)**. Core operation does not require OTel. When enabling FP-4 in future, apply the same metadata-only discipline to exported metric labels and span attributes.

## Operator dashboards

Operator UI is specified in OSS [Dashboard spec](dashboard-spec.md) but implemented in the private ArcFlow-Dashboard repository. Dashboard v1 UI is deferred (FP-3.01). Until exit criteria pass in private repo CI, operators use admin API, CLI trace, and SQL against `arcflow_trace_events`.

## Review checklist

Before shipping a new trace event or debug endpoint:

1. Does any field embed user or model text?
2. Does any field embed tool arguments or retrieval chunks?
3. Can this event reach the browser via Relay trace poll?
4. Do logs duplicate trace with richer content (forbidden)?

If any answer is yes, redesign the field or gate it behind localhost debug with `ARCFLOW_DEBUG`.

## Related pages

- [Architecture overview](architecture-overview.md) for Relay and trace poll flow
- [Maturity and known gaps](maturity-and-known-gaps.md) for FP-4 (OTel)
- [security/sec-1-compliance.md](../security/sec-1-compliance.md) for auth and webhook hardening
