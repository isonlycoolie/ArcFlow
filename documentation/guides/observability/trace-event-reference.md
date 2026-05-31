**Audience:** `[developer]` `[compliance]`

# Trace event reference

Complete reference for engine `TraceEventKind` variants emitted by `arcflow-core`. Each event serializes as `{ "kind": "<Name>", ...fields }` with PascalCase kind tags.

**SEC-1:** All listed events are metadata-only in storage and HTTP export. No prompt text, tool payloads, or chunk content.

Source of truth: `runtime/arcflow-core/src/tracing/events.rs`. Normative names: [contracts/normative/observability/trace-events-v1.md](../../../contracts/normative/observability/trace-events-v1.md).

**Total engine kinds:** 37 in `events.rs` (capabilities Appendix D also lists RCS graph kinds that may appear in extended exports).

## Workflow lifecycle

### WorkflowStarted

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | Execution UUID | Safe |
| `workflow_name` | string | Declared workflow name | Safe |
| `step_count` | usize | Linear step count or graph step count | Safe |

**Emitter:** workflow engine at run start.

### WorkflowCompleted

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | Execution UUID | Safe |
| `duration_ms` | u64 | Total wall time | Safe |
| `total_tokens` | TokenUsage | Aggregate input/output/total counts | Safe |

**Emitter:** workflow engine on success.

### WorkflowFailed

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | Execution UUID | Safe |
| `duration_ms` | u64 | Time until failure | Safe |
| `failed_step_index` | usize optional | Zero-based step index | Safe |
| `error_code` | string | RCS error code | Safe |

**Emitter:** workflow engine on terminal failure.

### WorkflowValidationFailed

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | Execution UUID | Safe |
| `reason` | string | Validation summary (must not embed user input) | Safe if bounded |

**Emitter:** validation layer before execution.

### WorkflowRecoveryStarted

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | New resume run id | Safe |
| `original_run_id` | string | Failed run id | Safe |
| `resume_from_step` | usize | Step index to resume | Safe |

**Emitter:** recovery subsystem.

### WorkflowRecoveryCompleted

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | Resume run id | Safe |
| `original_run_id` | string | Source failed run | Safe |
| `steps_re_executed` | usize | Steps run on resume | Safe |

**Emitter:** recovery subsystem.

## Step lifecycle

### StepStarted

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | Execution UUID | Safe |
| `step_id` | string | Step UUID | Safe |
| `step_index` | usize | Zero-based order | Safe |
| `agent_name` | string | Agent name from definition | Safe if not PII |
| `agent_role` | string | Agent role | Safe |

**Emitter:** step scheduler.

### StepCompleted

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | Execution UUID | Safe |
| `step_id` | string | Step UUID | Safe |
| `step_index` | usize | Step order | Safe |
| `duration_ms` | u64 | Step wall time | Safe |
| `tokens` | TokenUsage | Step token counts | Safe |
| `output_size_bytes` | usize | Output byte size, not text | Safe |

**Emitter:** step executor.

### StepFailed

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | Execution UUID | Safe |
| `step_id` | string | Step UUID | Safe |
| `step_index` | usize | Step order | Safe |
| `duration_ms` | u64 | Time until failure | Safe |
| `error_code` | string | RCS code | Safe |
| `error_message` | string | Safe summary text | Safe if no user content |

**Emitter:** step executor.

### StateCommitted

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | Execution UUID | Safe |
| `step_id` | string | Step UUID | Safe |
| `committed_step_count` | usize | Recovery checkpoint depth | Safe |

**Emitter:** recovery / state store.

### StepFallbackActivated

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | Execution UUID | Safe |
| `step_id` | string | Step UUID | Safe |
| `primary_agent_name` | string | Primary agent | Safe |
| `fallback_agent_name` | string | Fallback agent | Safe |

**Emitter:** step fallback resolver.

## Agent and provider

### AgentInvoked

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | Execution UUID | Safe |
| `step_id` | string | Step UUID | Safe |
| `agent_name` | string | Agent name | Safe if not PII |
| `input_size_bytes` | usize | Input byte size | Safe |

**Emitter:** agent runtime.

### AgentResponseReceived

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | Execution UUID | Safe |
| `step_id` | string | Step UUID | Safe |
| `agent_name` | string | Agent name | Safe |
| `output_size_bytes` | usize | Response byte size | Safe |

**Emitter:** agent runtime.

### TokensConsumed

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | Execution UUID | Safe |
| `step_id` | string | Step UUID | Safe |
| `agent_name` | string | Agent name | Safe |
| `tokens` | TokenUsage | Counts | Safe |

**Emitter:** agent / billing hook.

### ProviderRequestSent

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | Execution UUID | Safe |
| `step_id` | string | Step UUID | Safe |
| `provider_id` | string | Provider slug | Safe |
| `model_id` | string | Model id | Safe |
| `max_tokens` | u32 | Request cap | Safe |
| `prompt_size_bytes` | usize | Prompt size, not content | Safe |

**Emitter:** provider adapter.

### ProviderResponseReceived

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | Execution UUID | Safe |
| `step_id` | string | Step UUID | Safe |
| `provider_id` | string | Provider slug | Safe |
| `model_id` | string | Model id | Safe |
| `tokens` | TokenUsage | Usage counts | Safe |
| `latency_ms` | u64 | Round-trip time | Safe |

**Emitter:** provider adapter.

### ProviderRateLimited

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | Execution UUID | Safe |
| `step_id` | string | Step UUID | Safe |
| `provider_id` | string | Provider slug | Safe |
| `retry_after_seconds` | u64 optional | Provider hint | Safe |

**Emitter:** provider adapter.

### ProviderError

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | Execution UUID | Safe |
| `step_id` | string | Step UUID | Safe |
| `provider_id` | string | Provider slug | Safe |
| `error_code` | string | Provider or RCS code | Safe |
| `error_message` | string | Safe summary | Safe if bounded |

**Emitter:** provider adapter.

## Tools

### ToolCallStarted

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | Execution UUID | Safe |
| `step_id` | string | Step UUID | Safe |
| `tool_name` | string | Registered tool name | Safe |
| `input_schema_hash` | string | Hash of input schema, not args | Safe |

**Emitter:** tool executor.

### ToolCallCompleted

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | Execution UUID | Safe |
| `step_id` | string | Step UUID | Safe |
| `tool_name` | string | Tool name | Safe |
| `duration_ms` | u64 | Tool runtime | Safe |
| `output_size_bytes` | usize | Result size, not payload | Safe |

**Emitter:** tool executor.

### ToolCallFailed

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | Execution UUID | Safe |
| `step_id` | string | Step UUID | Safe |
| `tool_name` | string | Tool name | Safe |
| `duration_ms` | u64 | Time until failure | Safe |
| `failure_reason` | string | Bounded reason | Safe if bounded |
| `error_code` | string | RCS code | Safe |

**Emitter:** tool executor.

### ToolInputValidationFailed

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | Execution UUID | Safe |
| `step_id` | string | Step UUID | Safe |
| `tool_name` | string | Tool name | Safe |
| `violation_description` | string | Schema violation summary | Safe if no raw args |

**Emitter:** tool validator.

## Memory

### MemoryWrite

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | Execution UUID | Safe |
| `step_id` | string | Step UUID | Safe |
| `agent_name` | string | Agent name | Safe |
| `memory_type` | string | Backend type slug | Safe |
| `key` | string | Memory key (avoid PII in keys) | Caution |
| `duration_ms` | u64 | Operation time | Safe |

**Emitter:** memory coordinator.

### MemoryRead

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | Execution UUID | Safe |
| `step_id` | string | Step UUID | Safe |
| `agent_name` | string | Agent name | Safe |
| `memory_type` | string | Backend type | Safe |
| `key` | string | Memory key | Caution |
| `hit` | bool | Cache hit | Safe |
| `duration_ms` | u64 | Operation time | Safe |

**Emitter:** memory coordinator.

### MemoryRetrieved

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | Execution UUID | Safe |
| `step_id` | string | Step UUID | Safe |
| `agent_name` | string | Agent name | Safe |
| `chunk_count` | usize | Number of chunks | Safe |
| `total_bytes` | usize | Aggregate chunk bytes | Safe |

**Emitter:** vector retrieval (RAG).

### MemoryDegraded

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | Execution UUID | Safe |
| `memory_type` | string | Backend type | Safe |
| `backend` | string | Backend id | Safe |
| `reason` | string | Degradation reason | Safe |

**Emitter:** memory coordinator.

### MemoryEvicted

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | Execution UUID | Safe |
| `memory_type` | string | Backend type | Safe |
| `key` | string | Evicted key | Caution |
| `eviction_reason` | string | Reason enum / slug | Safe |

**Emitter:** memory eviction policy.

## Reliability

### RetryAttempted

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | Execution UUID | Safe |
| `step_id` | string | Step UUID | Safe |
| `attempt_number` | u32 | Current attempt | Safe |
| `max_attempts` | u32 | Configured cap | Safe |
| `backoff_ms` | u64 | Wait before retry | Safe |
| `trigger_error_code` | string | Code that triggered retry | Safe |

**Emitter:** retry policy.

### RetryExhausted

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | Execution UUID | Safe |
| `step_id` | string | Step UUID | Safe |
| `total_attempts` | u32 | Attempts made | Safe |
| `last_error_code` | string | Final error code | Safe |

**Emitter:** retry policy.

### TimeoutEnforced

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | Execution UUID | Safe |
| `step_id` | string | Step UUID | Safe |
| `timeout_type` | string | `workflow` or `step` | Safe |
| `configured_ms` | u64 | Limit | Safe |
| `elapsed_ms` | u64 | Observed elapsed | Safe |

**Emitter:** timeout enforcer.

### TraceStorageWarning

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | Execution UUID | Safe |
| `events_dropped` | u32 | Dropped event count | Safe |
| `capacity_limit` | u32 | Store capacity | Safe |

**Emitter:** trace ring buffer.

## Streaming

### StreamChunkReceived

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | Execution UUID | Safe |
| `step_id` | string | Step UUID | Safe |
| `chunk_bytes` | usize | Chunk size | Safe |

**Emitter:** streaming provider adapter.

### TokenEmitted

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | Execution UUID | Safe |
| `step_id` | string | Step UUID | Safe |
| `completion_token_delta` | u32 | Completion tokens in delta | Safe |
| `prompt_token_delta` | u32 | Prompt tokens in delta | Safe |

**Emitter:** streaming provider adapter. Browser poll uses this for progress UX.

## External bindings

### ExternalBindingStarted

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | Execution UUID | Safe |
| `binding_id` | string | Binding id | Safe |
| `step_id` | string | Attached step UUID | Safe |
| `mode` | string | `async_callback` or `sync_tool` | Safe |

**Emitter:** external monitor at step completion.

### ExternalBindingCompleted

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | Execution UUID | Safe |
| `binding_id` | string | Binding id | Safe |
| `step_id` | string | Step UUID | Safe |
| `duration_ms` | u64 | Wait duration | Safe |

**Emitter:** external monitor on success callback.

### ExternalBindingFailed

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | Execution UUID | Safe |
| `binding_id` | string | Binding id | Safe |
| `step_id` | string | Step UUID | Safe |
| `error_code` | string | Outcome or RCS code | Safe |
| `status` | string | Outcome status string | Safe |

**Emitter:** external monitor on failed callback.

### ExternalRecoveryTriggered

| Field | Type | Meaning | SEC-1 |
|-------|------|---------|-------|
| `run_id` | string | Execution UUID | Safe |
| `binding_id` | string | Binding id | Safe |
| `action` | string | Recovery action name | Safe |
| `attempt_number` | u32 | Retry attempt | Safe |

**Emitter:** external recovery policy.

## RCS graph kinds (extended exports)

These kinds are documented in RCS types and may appear alongside engine events in graph workflow exports. They follow the same SEC-1 rule (metadata only):

| Event | Typical fields |
|-------|----------------|
| `GraphNodeStarted` | run_id, node_id, iteration |
| `GraphNodeCompleted` | run_id, node_id, duration_ms |
| `GraphIterationLimitReached` | run_id, node_id, limit |

Confirm availability in your runtime version via a sample graph trace export.

## TokenUsage struct

Nested in multiple events:

| Field | Meaning | SEC-1 |
|-------|---------|-------|
| `input` | Prompt token count | Safe |
| `output` | Completion token count | Safe |
| `total` | Sum | Safe |

## Example full trace (metadata only)

```json
[
  { "kind": "WorkflowStarted", "run_id": "r1", "workflow_name": "demo", "step_count": 2 },
  { "kind": "StepStarted", "run_id": "r1", "step_id": "s1", "step_index": 0, "agent_name": "a1", "agent_role": "Analyst" },
  { "kind": "ProviderRequestSent", "run_id": "r1", "step_id": "s1", "provider_id": "openai", "model_id": "gpt-4o-mini", "max_tokens": 1024, "prompt_size_bytes": 512 },
  { "kind": "ProviderResponseReceived", "run_id": "r1", "step_id": "s1", "provider_id": "openai", "model_id": "gpt-4o-mini", "tokens": { "input": 120, "output": 45, "total": 165 }, "latency_ms": 890 },
  { "kind": "StepCompleted", "run_id": "r1", "step_id": "s1", "step_index": 0, "duration_ms": 920, "tokens": { "input": 120, "output": 45, "total": 165 }, "output_size_bytes": 180 },
  { "kind": "WorkflowCompleted", "run_id": "r1", "duration_ms": 950, "total_tokens": { "input": 120, "output": 45, "total": 165 } }
]
```

## Related pages

- [Execution traces](execution-traces.md) for SDK and HTTP access patterns
- [SEC-1 rules](sec-1-rules.md) for audit checklist
- [SDK streaming](../streaming/sdk-streaming.md) for stream event UX vs trace storage

## Source

Derived from [ARCFLOW-FULL-CAPABILITIES-REFERENCE.md](../../../docs/_draft/ARCFLOW-FULL-CAPABILITIES-REFERENCE.md) Appendix D; Sprint 5 TraceEventKind enum; Appendix K K-02, K-13, K-20; `runtime/arcflow-core/src/tracing/events.rs`.
