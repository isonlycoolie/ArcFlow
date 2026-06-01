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
