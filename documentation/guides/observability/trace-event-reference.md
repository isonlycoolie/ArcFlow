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
