
# SEC-1 compliance

SEC-1 is ArcFlow's mandatory trace data policy. Traces and persisted trace events describe **what happened** (identifiers, timings, sizes, error codes) without exposing **what was said** (prompts, completions, tool payloads, retrieved chunk text). This page is the authoritative compliance reference for audit teams.

Product context: [SEC-1 and data safety](../concepts/sec-1-and-data-safety.md). Implementation guide: [SEC-1 rules](../guides/observability/sec-1-rules.md).

Normative contract: [Trace events (normative)](../contracts/trace-events-normative.md).

## The SEC-1 rule

Traces and persisted trace events MUST NOT contain:

- Prompt or completion text
- Tool argument or result payloads
- Retrieved chunk text
- PII unless explicitly allowed by deployment policy

Allowed metadata includes: ids, names, roles, byte sizes, token counts, durations, error codes, schema hashes.

## Where SEC-1 applies

| Surface | SEC-1 scope |
|---------|-------------|
| SDK in-memory trace on `RunResult` | Yes (export JSON) |
| `GET /v1/runs/{id}/trace` | Yes |
| Relay `GET .../runs/{id}/trace` | Yes (browser reachable) |
| Postgres `arcflow_trace_events` | Yes (migration 000005) |
| VS Code timeline from export | Yes |
| OTel span attributes (FP-4 alpha) | Yes when enabled |
| Application logs | Policy: do not duplicate traces with richer content |

Run **result** payloads on `GET /v1/runs/{id}` may include final output text under your product policy. SEC-1 governs trace events specifically, not every API field.

## Runtime enforcement

The engine emits SEC-1 events from the tracing layer. Examples:

| Event | SEC-1 safe fields | Forbidden |
|-------|-------------------|-----------|
| `ProviderRequestSent` | `prompt_size_bytes`, `model_id` | Prompt text |
| `ToolCallStarted` | `input_schema_hash` | Tool arguments |
| `MemoryRetrieved` | `chunk_count`, `total_bytes` | Chunk bodies |
| `StreamChunkReceived` | `chunk_bytes` (size) | Chunk text in storage |

Persisted rows in `arcflow_trace_events` store JSON event payloads only. No separate prompt/completion columns exist by design.

### Postgres table structure (conceptual)

Migration `20240531000005` creates `arcflow_trace_events` with metadata JSON per event. Audit expects:

- `run_id`, `sequence`, `kind`, serialized metadata fields
- No columns for raw LLM I/O

Inspect with:

```sql
SELECT run_id, kind, payload
FROM arcflow_trace_events
WHERE run_id = 'your-run-uuid'
ORDER BY sequence
LIMIT 20;
```

Replace `payload` with actual column names from migration SQL in `server/arcflow-server/migrations/`.

## Compliance audit procedure

1. **Sample export:** `GET /v1/runs/{id}/trace` for a RAG and tool-heavy run.
2. **Secret grep:** Search JSON for `@`, `sk-`, `Bearer `, large base64 blobs.
3. **Field review:** Compare each event kind to [Trace events normative](../contracts/trace-events-normative.md) SEC-1 column.
4. **Browser path:** Repeat via Relay trace URL with site token.
5. **Logs:** Confirm application logs do not duplicate traces with prompt text.
6. **OTel (if enabled):** Inspect Jaeger/Tempo for forbidden attribute keys.

### Audit SQL (pattern)

```sql
-- Events for a run (adjust table/column names to your migration)
SELECT kind, payload::text
FROM arcflow_trace_events
WHERE run_id = $1;
```

Manual review: no substring matches for user-provided essay content, API keys, or email addresses in trace payloads.

## Developer and operator responsibilities

| Role | Responsibility |
|------|----------------|
| Workflow author | Avoid PII in `agent_name`, `tool_name` |
| Platform | Keep `ARCFLOW_DEBUG=false` in production |
| Operator | Do not log ingest body to analytics |
| Integrator | Do not attach raw payloads to custom trace exporters |

## Before approving new observability features

1. Does any new field embed user or model text?
2. Does any field embed tool arguments or retrieval chunks?
3. Can this event reach the browser via Relay trace poll?
4. Do logs duplicate trace with richer content?

If any answer is yes, reject or gate behind localhost debug.

## Related pages

- [Trace events normative](../contracts/trace-events-normative.md)
- [Webhook HMAC](webhook-hmac.md) (external payloads separate from traces)
- [Self-hosted security](self-hosted-security.md)
