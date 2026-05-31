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
