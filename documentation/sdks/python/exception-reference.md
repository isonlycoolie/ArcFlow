# Python SDK exception reference

**Audience:** `[developer]`

ArcFlow Python exceptions inherit from `ArcFlowError`. Messages follow the format `[ArcFlow] <what happened>. <what to do>.` when raised by SDK validation layers. Native runtime errors are mapped into the same hierarchy before they reach your code.

## Hierarchy

```
ArcFlowError
├── WorkflowConfigurationError
├── WorkflowExecutionError
│   ├── RetryExhaustedError
│   └── WorkflowTimeoutError
├── ToolConfigurationError
├── ToolExecutionError
├── MemoryConfigurationError
├── MemoryOperationError
├── ProviderConfigurationError
├── ProviderExecutionError
├── InfrastructureUnavailableError
├── TraceNotFoundError
├── TraceStorageWarning
├── HumanRejectedError          (arcflow.hitl)
└── WorkflowInterruptedError    (arcflow.hitl)
```

## Configuration errors (fix before run)

These indicate invalid workflow, agent, tool, memory, or provider definitions. No run is started or the failure happens before native execution begins.

### WorkflowConfigurationError

| Trigger | Remediation |
|---------|-------------|
| Empty workflow name | Pass non-empty `Workflow("name")` |
| `step()` on graph workflow | Use `node()` when `graph=True` |
| `node()` on linear workflow | Pass `graph=True` to constructor |
| Empty run input | Pass non-empty string to `run()` |
| No steps or nodes | Add at least one step or node |
| Invalid retry, timeout, recovery config | Check numeric bounds and call order (before `run()`) |
| `run_stream()` with remote `runtime=` | Use in-process workflow or non-streaming remote run |
| `resume()` without `enable_recovery()` | Call `enable_recovery()` first |
| Fallback agent not in prior steps | Register fallback in an earlier `step()` |

### ToolConfigurationError

| Trigger | Remediation |
|---------|-------------|
| Empty tool name or description | Provide non-empty strings |
| Invalid `input_schema` | Pass JSON-serializable JSON Schema dict |
| Non-callable `execute` | Pass `(dict) -> str` callable |
| Invalid `timeout_seconds` | Must be positive |

### MemoryConfigurationError

| Trigger | Remediation |
|---------|-------------|
| Missing namespace for persistent/vector | Set `namespace=` on `MemoryConfig` |
| Invalid retrieval or chunking values | See [API reference](api-reference.md) bounds |
| VectorStore called without namespace | Pass non-empty namespace to ingest/search |

### ProviderConfigurationError

| Trigger | Remediation |
|---------|-------------|
| Empty model string | e.g. `OpenAI(model="gpt-4o")` |
| Temperature outside 0.0 to 1.0 | Adjust temperature |
| `max_tokens` below 1 | Use positive token limit |

## Execution errors (during run)

### WorkflowExecutionError

A step failed during execution.

| Attribute | Type | Meaning |
|-----------|------|---------|
| `run_id` | `str \| None` | Run UUID when available |
| `failed_step` | `str \| None` | Step id that failed |

Check trace via `workflow.trace()` or CLI `arcflow trace <run_id>`.

### RetryExhaustedError

Subclass of `WorkflowExecutionError`. All retry attempts exhausted.

| Attribute | Type |
|-----------|------|
| `attempts_made` | `int` |
| `last_error_code` | `str \| None` |

Review retry policy on `workflow.retry()` and upstream provider errors.

### WorkflowTimeoutError

Subclass of `WorkflowExecutionError`. Workflow or step timeout enforced.

| Attribute | Type |
|-----------|------|
| `timeout_type` | `str` |
| `configured_seconds` | `float` |
| `elapsed_seconds` | `float` |

Increase `timeout()` / `step_timeout()` or reduce work per step.

### ToolExecutionError

Tool callable raised or native tool layer failed.

| Attribute | Type |
|-----------|------|
| `tool_name` | `str \| None` |
| `run_id` | `str \| None` |
| `failed_step` | `str \| None` |

### MemoryOperationError

Memory read or write failed at runtime. Verify backend URL, migrations, and namespace.

### ProviderExecutionError

LLM provider call failed.

| Attribute | Type |
|-----------|------|
| `provider_id` | `str \| None` |
| `run_id` | `str \| None` |
| `failed_step` | `str \| None` |

Verify API key env vars and model name.

### InfrastructureUnavailableError

Optional backend unreachable or URL unset.

| Attribute | Type |
|-----------|------|
| `backend` | `str \| None` |
| `suggestion` | `str \| None` |

Common case: Postgres or Qdrant not running. Start Docker stack and export env vars from [installation](installation.md).

## Observability errors

### TraceNotFoundError

No trace for requested run.

| Cause | Fix |
|-------|-----|
| `trace()` before `run()` | Call `run()` first |
| Run evicted from in-process store | Re-run or use CLI with persisted trace |

### TraceStorageWarning

Trace store dropped events at capacity. Not always fatal; check `TraceResult.warnings`.

| Attribute | Type |
|-----------|------|
| `events_dropped` | `int` |
| `run_id` | `str \| None` |

Reduce trace volume or increase store limits in runtime config.

## HITL errors

### WorkflowInterruptedError

Workflow paused waiting for human approval. Expected control flow for HITL, not necessarily a bug.

| Attribute | Type |
|-----------|------|
| `run_id` | `str` |
| `approval_key` | `str` |
| `expires_at` | `str \| None` |

Resume with `workflow.resume_with_approval(run_id, approval_key, approved=True)`.

### HumanRejectedError

Human rejected the approval request.

| Attribute | Type |
|-----------|------|
| `approval_key` | `str \| None` |

## External callback errors

`report_outcome()` raises standard Python exceptions outside the ArcFlow hierarchy:

| Exception | Cause |
|-----------|-------|
| `ValueError` | Missing `ARCFLOW_SERVER_API_KEY` or `ARCFLOW_WEBHOOK_SECRET` |
| `RuntimeError` | HTTP error or network failure from callback POST |

## Quick lookup table

| Exception | Phase | Typical fix |
|-----------|-------|-------------|
| `WorkflowConfigurationError` | Pre-run | Fix workflow definition |
| `WorkflowExecutionError` | Run | Inspect trace, failed step |
| `ToolConfigurationError` | Pre-run | Fix tool schema or callable |
| `ToolExecutionError` | Run | Fix tool implementation |
| `MemoryConfigurationError` | Pre-run | Fix `MemoryConfig` |
| `MemoryOperationError` | Run | Fix backend connectivity |
| `ProviderConfigurationError` | Pre-run | Fix provider constructor args |
| `ProviderExecutionError` | Run | Fix API key or model |
| `InfrastructureUnavailableError` | Run | Start Postgres/Qdrant |
| `TraceNotFoundError` | Post-run | Run workflow first |
| `TraceStorageWarning` | Post-run | Reduce event volume |
| `RetryExhaustedError` | Run | Adjust retry or root cause |
| `WorkflowTimeoutError` | Run | Increase timeout |
| `WorkflowInterruptedError` | Run (HITL) | Approve or reject |
| `HumanRejectedError` | Run (HITL) | Handle rejection path |

## Source

`sdk-python/arcflow/exceptions.py`, `sdk-python/arcflow/hitl.py`, `sdk-python/README.md` (Errors section); capabilities reference §16.
