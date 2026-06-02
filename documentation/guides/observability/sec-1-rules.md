
# Trace data policy rules (implementation guide)

The ArcFlow trace data policy is mandatory for production deployments. Observability outputs describe **what happened** (ids, timings, sizes, codes) without exposing **what was said** (prompts, completions, tool payloads, retrieved text). This page is the implementation and audit guide; [Trace data policy](../../concepts/sec-1-and-data-safety.md) covers product context.

The rule applies everywhere traces leak: SDK memory, `GET /v1/runs/{id}/trace`, Relay trace poll, Postgres `arcflow_trace_events`, VS Code timeline exports, and optional OTel span attributes (OpenTelemetry export (alpha)).

## Allowed field categories

| Category | Examples | trace data policy class |
|----------|----------|-------------|
| Identifiers | `run_id`, `step_id`, `trace_id`, workflow name | Safe |
| Names / roles | `agent_name`, `agent_role`, `tool_name`, `binding_id` | Safe if not user PII |
| Sizes | `prompt_size_bytes`, `output_size_bytes`, `input_size_bytes`, `chunk_bytes` | Safe |
| Counts | `chunk_count`, `total_bytes`, `step_count`, token deltas | Safe |
| Tokens struct | `tokens.input`, `tokens.output`, `tokens.total` | Safe |
| Durations | `duration_ms`, `latency_ms`, `backoff_ms` | Safe |
| Error codes | workflow specification `error_code`, bounded `error_message` | Safe when no embedded user text |
| Schema hashes | `input_schema_hash` | Safe |
| Provider metadata | `provider_id`, `model_id`, `retry_after_seconds` | Safe |
| Status enums | `status`, `mode`, `action` on external events | Safe |

## Forbidden field categories

| Category | Examples | Why forbidden |
|----------|----------|---------------|
| LLM content | Prompt text, system instructions as sent, completion text | Conversation exfiltration via trace poll |
| Tool I/O | Tool argument JSON, tool result payloads | Same as logging raw API bodies |
| RAG content | Retrieved chunk text, embedding query strings | Knowledge base leakage |
| Credentials | API keys, webhook secrets, tokens | Credential exposure |
| Unbounded PII | Full names, emails, phone numbers in trace fields | Compliance and browser exposure |

If a field could appear in a browser network tab via Relay, treat it as forbidden unless explicitly approved by your deployment policy.

## Runtime enforcement

Engine emission is implemented in `runtime/arcflow-core/src/tracing/events.rs`. Event variants only carry metadata fields. For example:

- `ProviderRequestSent` records `prompt_size_bytes`, not prompt text.
- `ToolCallStarted` records `input_schema_hash`, not arguments.
- `MemoryRetrieved` records `chunk_count` and `total_bytes`, not chunk bodies.
- `TokenEmitted` records numeric deltas, not token strings in stored traces.

Streaming at the SDK layer may expose `event.text` to **your** process during `run_stream()`, but persisted trace rows and HTTP exports remain count-based.

## Developer responsibilities

trace data policy is not only an engine concern. Workflow authors must avoid:

| Mistake | Trace impact |
|---------|--------------|
| Putting user emails in `agent_name` or `tool_name` | Names appear in every step event |
| Logging `result.trace_events` alongside raw prompts in app logs | Bypasses trace policy |
| Returning retrieved chunks in tool names | Appears in `ToolCallStarted` |
| Custom debug flags that dump envelopes to stdout | Out of band leakage |

Use neutral agent and tool names. Keep PII in your application database, not in the workflow specification identifiers that emit to trace.

## Run results vs traces

`GET /v1/runs/{id}` **result** payload may include final `output` text appropriate to your product policy. The trace data policy governs **trace events**, not every API field. HITL interrupt objects expose `approval_key` and `expires_at`, not transcripts.

Document separately which API fields your compliance team treats as sensitive.

## Audit procedure

1. **Sample export**: Pull `GET /v1/runs/{id}/trace` for a RAG and tool-heavy run.
2. **Greppable secrets**: Search JSON for `@`, `sk-`, `Bearer `, large base64 blobs.
3. **Field review**: Compare each event kind to [trace event reference](trace-event-reference.md) Trace policy column.
4. **Browser path**: Repeat via Relay trace URL with a site token.
5. **Logs**: Confirm application logs do not duplicate traces with richer content.
6. **OTel (if OpenTelemetry metrics export enabled)**: Inspect span attributes in Jaeger for prompt or completion keys.

Normative contract: [Trace events (normative)](../../contracts/trace-events-normative.md).

## Safe trace excerpt

```json
[
 { "kind": "WorkflowStarted", "run_id": "r1", "workflow_name": "demo", "step_count": 2 },
 { "kind": "ProviderRequestSent", "run_id": "r1", "step_id": "s1", "provider_id": "openai", "model_id": "gpt-4o-mini", "prompt_size_bytes": 512 },
 { "kind": "MemoryRetrieved", "run_id": "r1", "step_id": "s1", "agent_name": "researcher", "chunk_count": 5, "total_bytes": 8192 },
 { "kind": "ToolCallCompleted", "run_id": "r1", "step_id": "s1", "tool_name": "search_kb", "duration_ms": 210, "output_size_bytes": 8192 },
 { "kind": "WorkflowCompleted", "run_id": "r1", "duration_ms": 950, "total_tokens": { "input": 120, "output": 45, "total": 165 } }
]
```

## Before shipping new observability features

Checklist for new events, debug endpoints, or dashboard widgets:

1. Does any field embed user or model text?
2. Does any field embed tool arguments or retrieval chunks?
3. Can this event reach the browser via Relay trace poll?
4. Do logs duplicate trace with richer content?

If any answer is yes, redesign the field or gate behind localhost debug with `ARCFLOW_DEBUG=true`.

## Related pages

- [Trace event reference](trace-event-reference.md) per-event trace data policy classification
- [Opentelemetry](opentelemetry.md) for OpenTelemetry metrics export span attribute rules
- [Webhook security](../external-integrations/webhook-security.md) for callback body logging
