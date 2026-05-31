**Audience:** `[compliance]` `[developer]`

# Trace events (normative)

Normative naming contract for ArcFlow engine trace event kinds. This is a reference document, not a tutorial. Each event serializes as `{ "kind": "<PascalCaseName>", ...fields }`.

**SEC-1 (absolute):** No LLM prompts/responses, tool input/output values, memory values, credentials, raw workflow input, or PII in stored or exported traces.

Source of truth in code: `runtime/arcflow-core/src/tracing/events.rs`.

Canonical normative file: [contracts/normative/observability/trace-events-v1.md](../../contracts/normative/observability/trace-events-v1.md). This documentation page aligns with Appendix D and supersedes stale sections where the repo copy lags engine behavior (K-20).

Tutorial-style reference: [Trace event reference](../guides/observability/trace-event-reference.md).

**Total engine event kinds:** 40 (K-02). RCS graph envelope kinds may appear in extended exports alongside engine events.

## Envelope

Every event shares correlation fields where applicable:

| Field | Type | Description |
|-------|------|-------------|
| `trace_id` | string | Same as run_id |
| `timestamp` | datetime UTC ms | Emission time |
| `sequence` | u64 | Monotonic per run |
| `kind` | string | PascalCase variant name |

## D.1 Workflow lifecycle

| Kind | Trigger | Fields | SEC-1 |
|------|---------|--------|-------|
| `WorkflowStarted` | Run begins | run_id, workflow_name, step_count | Safe |
| `WorkflowCompleted` | All steps success | run_id, duration_ms, total_tokens | Safe |
| `WorkflowFailed` | Terminal failure | run_id, duration_ms, failed_step_index?, error_code | Safe |
| `WorkflowValidationFailed` | Pre-run validation fail | run_id, reason | Safe if reason has no user text |
| `WorkflowRecoveryStarted` | Resume begins | run_id, original_run_id, resume_from_step | Safe |
| `WorkflowRecoveryCompleted` | Resume finishes | run_id, original_run_id, steps_re_executed | Safe |

## D.2 Step lifecycle

| Kind | Trigger | Fields | SEC-1 |
|------|---------|--------|-------|
| `StepStarted` | Before agent for step | run_id, step_id, step_index, agent_name, agent_role | Safe if names not PII |
| `StepCompleted` | Step success | run_id, step_id, step_index, duration_ms, tokens, output_size_bytes | Safe |
| `StepFailed` | Step failure | run_id, step_id, step_index, duration_ms, error_code, error_message | Safe if message bounded |
| `StateCommitted` | Output committed | run_id, step_id, committed_step_count | Safe |
| `StepFallbackActivated` | Fallback agent used | run_id, step_id, primary_agent_name, fallback_agent_name | Safe |

## D.3 Agent and provider

| Kind | Trigger | Fields | SEC-1 |
|------|---------|--------|-------|
| `AgentInvoked` | Agent processing starts | run_id, step_id, agent_name, input_size_bytes | Safe |
| `AgentResponseReceived` | Agent output ready | run_id, step_id, agent_name, output_size_bytes | Safe |
| `TokensConsumed` | Token counts known | run_id, step_id, agent_name, tokens | Safe |
| `ProviderRequestSent` | LLM request dispatched | run_id, step_id, provider_id, model_id, max_tokens, prompt_size_bytes | Safe |
| `ProviderResponseReceived` | LLM response received | run_id, step_id, provider_id, model_id, tokens, latency_ms | Safe |
| `ProviderRateLimited` | Provider 429 | run_id, step_id, provider_id, retry_after_seconds? | Safe |
| `ProviderError` | Provider failure | run_id, step_id, provider_id, error_code, error_message | Safe if bounded |

## D.4 Tools

| Kind | Trigger | Fields | SEC-1 |
|------|---------|--------|-------|
| `ToolCallStarted` | Tool dispatch | run_id, step_id, tool_name, input_schema_hash | Safe |
| `ToolCallCompleted` | Tool success | run_id, step_id, tool_name, duration_ms, output_size_bytes | Safe |
| `ToolCallFailed` | Tool failure | run_id, step_id, tool_name, duration_ms, failure_reason, error_code | Safe |
| `ToolInputValidationFailed` | Schema validation fail | run_id, step_id, tool_name, violation_description | Safe if no arg values |

## D.5 Memory

| Kind | Trigger | Fields | SEC-1 |
|------|---------|--------|-------|
| `MemoryWrite` | Write to memory | run_id, step_id, agent_name, memory_type, key, duration_ms | Safe |
| `MemoryRead` | Read from memory | run_id, step_id, agent_name, memory_type, key, hit, duration_ms | Safe |
| `MemoryRetrieved` | RAG retrieval | run_id, step_id, agent_name, chunk_count, total_bytes | Safe |
| `MemoryDegraded` | Backend degraded | run_id, memory_type, backend, reason | Safe |
| `MemoryEvicted` | Eviction | run_id, memory_type, key, eviction_reason | Safe |

## D.6 Reliability

| Kind | Trigger | Fields | SEC-1 |
|------|---------|--------|-------|
| `RetryAttempted` | Retry scheduled | run_id, step_id, attempt_number, max_attempts, backoff_ms, trigger_error_code | Safe |
| `RetryExhausted` | Retries exhausted | run_id, step_id, total_attempts, last_error_code | Safe |
| `TimeoutEnforced` | Timeout fired | run_id, step_id, timeout_type, configured_ms, elapsed_ms | Safe |
| `TraceStorageWarning` | Trace buffer pressure | run_id, events_dropped, capacity_limit | Safe |

## D.7 Streaming

| Kind | Trigger | Fields | SEC-1 |
|------|---------|--------|-------|
| `StreamChunkReceived` | Stream chunk (size only in trace) | run_id, step_id, chunk_bytes | Safe |
| `TokenEmitted` | Token delta accounting | run_id, step_id, completion_token_delta, prompt_token_delta | Safe |

