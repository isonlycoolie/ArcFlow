**Audience:** `[developer]` `[platform]`

# Error codes

Complete reference for RCS `ErrorCode` variants (16 total, K-03). Defined in `runtime/arcflow-core/src/rcs/types.rs`.

Errors surface in workflow results, trace events (`WorkflowFailed`, `StepFailed`), and HTTP API responses on the server.

SDK details: [Python exception reference](../sdks/python/exception-reference.md), [TypeScript exception reference](../sdks/typescript/exception-reference.md).

## Workflow definition errors

### WorkflowNotFound

| Aspect | Value |
|--------|-------|
| Typical cause | Registry miss for `workflow_ref` name/version range |
| HTTP (server) | 404 |
| Python | `WorkflowExecutionError` / infrastructure wrapper |
| TypeScript | `WorkflowExecutionError` |
| Handling | Verify registry publish; check semver range |

### InvalidWorkflowDefinition

| Aspect | Value |
|--------|-------|
| Typical cause | Schema validation or `validate_graph` failure |
| HTTP (server) | 400 |
| Python | `WorkflowConfigurationError` |
| TypeScript | `WorkflowConfigurationError` |
| Handling | Fix RCS JSON against [v1.schema.json](../../contracts/normative/rcs/v1.schema.json) |

### UnsupportedRcsVersion

| Aspect | Value |
|--------|-------|
| Typical cause | Workflow declares future RCS version |
| HTTP (server) | 400 |
| Python | `WorkflowConfigurationError` |
| TypeScript | `WorkflowConfigurationError` |
| Handling | Upgrade runtime or downgrade workflow version |

## Execution errors

### StepExecutionFailed

| Aspect | Value |
|--------|-------|
| Typical cause | Agent or step terminal failure |
| HTTP (server) | 422 or 500 |
| Python | `WorkflowExecutionError` |
| TypeScript | `WorkflowExecutionError` |
| Handling | Inspect trace; fix agent config or input |

### ToolExecutionFailed

| Aspect | Value |
|--------|-------|
| Typical cause | Tool handler returned error |
| HTTP (server) | 422 |
| Python | `ToolExecutionError` |
| TypeScript | `ToolExecutionError` |
| Handling | Fix tool implementation or inputs at workflow design time |

### InternalError

| Aspect | Value |
|--------|-------|
| Typical cause | Unexpected engine error |
| HTTP (server) | 500 |
| Python | `WorkflowExecutionError` |
| TypeScript | `WorkflowExecutionError` |
| Handling | Retry; escalate with run_id and trace export |

## Provider errors

### ProviderError

| Aspect | Value |
|--------|-------|
| Typical cause | LLM API error (4xx/5xx from provider) |
| HTTP (server) | 502 |
| Python | `ProviderExecutionError` |
| TypeScript | `ProviderExecutionError` |
| Handling | Check provider status, key, model id |

### EmbeddingError

| Aspect | Value |
|--------|-------|
| Typical cause | Embedding provider failure during ingest/RAG |
| HTTP (server) | 502 |
| Python | `ProviderExecutionError` / memory errors |
| TypeScript | Provider or memory error types |
| Handling | Verify `ARCFLOW_EMBEDDING_PROVIDER` and API key |
