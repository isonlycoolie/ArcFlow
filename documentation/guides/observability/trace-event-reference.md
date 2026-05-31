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

