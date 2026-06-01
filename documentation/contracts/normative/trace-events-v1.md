# TRACE-EVENT-SCHEMA v1

**Status:** Draft for CAA approval (Sprint 5 Phase 0.2)  
**Owner:** Observability Agent (OBA)  
**Consumers:** `arcflow-core/src/tracing/events.rs`, `ExecutionTraceBuilder`, Python `trace_bridge`, CLI `arcflow trace`

## Principles

1. Every workflow run emits a complete trace by construction (not optional).
2. Events are typed structs â€” never opaque `data` / `value` / `extra` bags.
3. **SEC-1 (absolute):** No LLM prompts/responses, tool input/output values, memory values, credentials, raw workflow input, or PII.
4. Timestamps: UTC, millisecond precision.
5. Correlation: `run_id` on every variant; `step_id` where step-scoped.

## Envelope: `TraceEvent`

| Field | Type | Description |
|-------|------|-------------|
| `trace_id` | string | Same as `run_id` / `WorkflowResult.run_id` |
| `timestamp` | datetime (UTC, ms) | Emission time |
| `sequence` | u64 | Monotonic per run (ordering tie-break) |
| `kind` | `TraceEventKind` | Payload (this document) |

---

## Workflow lifecycle

### TraceEventKind::WorkflowStarted

**Emitted by:** `WorkflowEngine`  
**Emitted when:** At the start of `execute()`, before any step runs.  
**Count per run:** Exactly once (first event).

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| run_id | string | Run UUID | `f47ac10b-...` |
| workflow_name | string | Declared workflow name | `research-pipeline` |
| step_count | usize | Steps in definition | `3` |

**What this tells a developer:** Execution began; how many steps were scheduled.

**NEVER contains:** Workflow input text, agent instructions, env secrets.

---

### TraceEventKind::WorkflowCompleted

**Emitted by:** `WorkflowEngine`  
**Emitted when:** All steps finish successfully.  
**Count per run:** Once (terminal on success).

| Field | Type | Description |
|-------|------|-------------|
| run_id | string | Run UUID |
| duration_ms | u64 | Wall-clock total |
| total_tokens | TokenUsage | Sum across steps |

**NEVER contains:** Final output text, memory contents.

---

### TraceEventKind::WorkflowFailed

**Emitted by:** `WorkflowEngine`  
**Emitted when:** Unrecoverable failure ends the run.  
**Count per run:** Once (terminal on failure).

| Field | Type | Description |
|-------|------|-------------|
| run_id | string | Run UUID |
| duration_ms | u64 | Elapsed until failure |
| failed_step_index | usize \| null | Failing step index, if known |
| error_code | string | Machine-readable code |

**NEVER contains:** Stack traces with user data, tool/memory payloads.

---

### TraceEventKind::WorkflowValidationFailed

**Emitted by:** `WorkflowEngine`  
**Emitted when:** Definition validation fails before execution.  
**Count per run:** Once (terminal).

| Field | Type | Description |
|-------|------|-------------|
| run_id | string | Run UUID |
| reason | string | Field/constraint names only |

**NEVER contains:** User-provided input strings used in validation.

---

## Step lifecycle

### TraceEventKind::StepStarted

**Emitted by:** `WorkflowEngine`  
**Emitted when:** Immediately before `AgentRuntime` for a step.  
**Count per run:** Once per executed step.

| Field | Type | Description |
|-------|------|-------------|
| run_id, step_id | string | Identifiers |
| step_index | usize | 0-based order |
| agent_name | string | Agent display name |

**NEVER contains:** Step input content, instructions body.

---

### TraceEventKind::StepCompleted

**Emitted by:** `WorkflowEngine`  
**Emitted when:** Step returns success.  
**Count per run:** Once per successful step.

| Field | Type | Description |
|-------|------|-------------|
| run_id, step_id | string | Identifiers |
| duration_ms | u64 | Step wall time |
| tokens | TokenUsage | Step token accounting |

---

### TraceEventKind::StepFailed

**Emitted by:** `WorkflowEngine`  
**Emitted when:** Step returns error.  
**Count per run:** Once per failed step (run may continue partial).

| Field | Type | Description |
|-------|------|-------------|
| run_id, step_id | string | Identifiers |
| duration_ms | u64 | Elapsed |
| error_code | string | Sanitized code |

---

### TraceEventKind::StateCommitted

**Emitted by:** `StateEngine`  
**Emitted when:** Step output committed to run state.  
**Count per run:** Once per committed step.

| Field | Type | Description |
|-------|------|-------------|
| run_id, step_id | string | Identifiers |
| output_size_bytes | usize | Size of committed output |

**NEVER contains:** Output content.

---

## Agent invocation

### TraceEventKind::AgentInvoked

**Emitted by:** `AgentRuntime`  
**Emitted when:** Agent processing starts for a step.  
**Count per run:** Once per step invocation.

| Field | Type | Description |
|-------|------|-------------|
| run_id, step_id | string | Identifiers |
| agent_name, agent_role | string | Identity |
| model_id | string | Model identifier string |

---

### TraceEventKind::AgentResponseReceived

**Emitted by:** `AgentRuntime`  
**Emitted when:** Agent produces output (stub or provider).  
**Count per run:** Once per step.

| Field | Type | Description |
|-------|------|-------------|
| run_id, step_id | string | Identifiers |
| response_size_bytes | usize | Output size only |

**NEVER contains:** Response text.

---

### TraceEventKind::TokensConsumed

**Emitted by:** `AgentRuntime`  
**Emitted when:** Token counts are known for a step.  
**Count per run:** Once per step (may coalesce with response).

| Field | Type | Description |
|-------|------|-------------|
| run_id, step_id | string | Identifiers |
| tokens | TokenUsage | prompt / completion / total |

**NEVER contains:** Token strings.

---

## Tool events

### TraceEventKind::ToolCallStarted

**Emitted by:** `ToolRuntime`  
**Emitted when:** Tool dispatch begins (after schema validation).  
**Count per run:** Once per tool invocation.

| Field | Type | Description |
|-------|------|-------------|
| run_id, step_id | string | Identifiers |
| tool_name | string | Registered tool name |
| input_schema_hash | string | SHA-256 of input schema |

**NEVER contains:** Tool argument values.

---

### TraceEventKind::ToolCallCompleted

**Emitted by:** `ToolRuntime`  
**Emitted when:** Tool returns success.  
**Count per run:** Once per successful tool call.

| Field | Type | Description |
|-------|------|-------------|
| run_id, step_id, tool_name | string | Identifiers |
| duration_ms | u64 | Wall time |
| output_size_bytes | usize | Output size |

---

### TraceEventKind::ToolCallFailed

**Emitted by:** `ToolRuntime`  
**Emitted when:** Tool execution or timeout fails.  
**Count per run:** Once per failed invocation.

| Field | Type | Description |
|-------|------|-------------|
| run_id, step_id, tool_name | string | Identifiers |
| duration_ms | u64 | Elapsed |
| failure_reason | string | `execution_error` \| `timeout` \| â€¦ |
| error_code | string | Sanitized code |

---

### TraceEventKind::ToolInputValidationFailed

**Emitted by:** `ToolRuntime`  
**Emitted when:** JSON Schema validation fails before execute.  
**Count per run:** Once per failed validation.

| Field | Type | Description |
|-------|------|-------------|
| run_id, step_id, tool_name | string | Identifiers |
| violation_description | string | JSON paths / constraints only |

**NEVER contains:** Actual input JSON values.

---

## Memory events

### TraceEventKind::MemoryWrite

**Emitted by:** Memory coordinator (all four backends)  
**Emitted when:** Write completes.  
**Count per run:** Once per write operation.

| Field | Type | Description |
|-------|------|-------------|
| run_id, step_id, agent_name | string | Context |
| memory_type | string | `session` \| `shared` \| `persistent` \| `vector` |
| key | string | Namespaced logical key |
| duration_ms | u64 | Operation latency |

**NEVER contains:** Written value bytes.

---

### TraceEventKind::MemoryRead

**Emitted by:** Memory coordinator  
**Emitted when:** Read completes.  
**Count per run:** Once per read operation.

| Field | Type | Description |
|-------|------|-------------|
| run_id, step_id, agent_name | string | Context |
| memory_type, key | string | Backend + key |
| hit | bool | Cache hit |
| duration_ms | u64 | Latency |

**NEVER contains:** Retrieved value.

---

### TraceEventKind::MemoryDegraded

**Emitted by:** Memory coordinator  
**Emitted when:** Backend unavailable; fallback used.  
**Count per run:** As needed.

| Field | Type | Description |
|-------|------|-------------|
| run_id | string | Run UUID |
| memory_type | string | Affected backend kind |
| backend | string | `redis` \| `postgresql` \| `qdrant` |
| reason | string | Sanitized reason |

---

### TraceEventKind::MemoryEvicted

**Emitted by:** Memory coordinator  
**Emitted when:** Key evicted (TTL or delete).  
**Count per run:** As needed.

| Field | Type | Description |
|-------|------|-------------|
| run_id | string | Run UUID |
| memory_type, key | string | Target |
| eviction_reason | string | `ttl_expiry` \| `explicit_delete` |

---

## Provider events (Sprint 5 schema; Sprint 6 wiring)

### TraceEventKind::ProviderRequestSent

**Emitted by:** Provider layer (stub counts in Sprint 5)  
**Emitted when:** Before LLM API call.  
**Count per run:** Per provider call.

| Field | Type | Description |
|-------|------|-------------|
| run_id, step_id | string | Identifiers |
| provider_id | string | Provider enum name |
| model_id | string | Model id |
| max_tokens_budget | u32 | Configured cap |

**NEVER contains:** Prompt text.

---

### TraceEventKind::ProviderResponseReceived

**Emitted when:** After LLM API returns.  

| Field | Type | Description |
|-------|------|-------------|
| run_id, step_id | string | Identifiers |
| duration_ms | u64 | RTT |
| tokens | TokenUsage | Usage counts |

**NEVER contains:** Completion text.

---

### TraceEventKind::ProviderRateLimited

| Field | Type | Description |
|-------|------|-------------|
| run_id, step_id | string | Identifiers |
| retry_after_ms | u64 | Hint from provider |

---

### TraceEventKind::ProviderError

| Field | Type | Description |
|-------|------|-------------|
| run_id, step_id | string | Identifiers |
| error_code | string | Sanitized provider error |

---

## System events

### TraceEventKind::TraceStorageWarning

**Emitted by:** `TraceEventEmitter` / `TraceStore`  
**Emitted when:** Per-run event cap reached; oldest events dropped.  
**Count per run:** As needed (warning, not fatal).

| Field | Type | Description |
|-------|------|-------------|
| run_id | string | Run UUID |
| events_dropped | u32 | Dropped since last warning |
| capacity_limit | u32 | `MAX_TRACE_EVENTS_PER_RUN` |

---

### TraceEventKind::RetryAttempted

**Emitted by:** Retry engine (variant defined Sprint 5; wired Sprint 7)  
**Emitted when:** Step retry scheduled.

| Field | Type | Description |
|-------|------|-------------|
| run_id, step_id | string | Identifiers |
| attempt_number | u32 | 1-based retry index |
| max_attempts | u32 | Policy cap |
| backoff_ms | u64 | Delay before retry |
| trigger_error_code | string | Sanitized trigger |

---

## Sprint 4 â†’ Sprint 5 migration

| Sprint 4 (RCS / emitter) | Sprint 5 |
|--------------------------|----------|
| `ToolExecuted` (single event) | `ToolCallStarted` + `ToolCallCompleted` \| `ToolCallFailed` |
| `MemoryRead` / `MemoryWrite` (JSON metadata blob) | Same kinds, structured fields per table above |
| No `sequence` on events | `sequence` required on envelope |
| Events in `rcs::types` | Canonical enum in `tracing::events`; RCS may re-export subset |

## Approval

- [ ] CAA â€” schema complete, SEC-1 satisfied  
- [ ] OBA â€” field names semantic, granularity approved  
- [ ] QAA â€” test matrix can be derived from this document  
