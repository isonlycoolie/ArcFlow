
# Error codes

Complete reference for RCS `ErrorCode` variants (16 total, K-03). Normative definitions align with [RCS schema](rcs-schema.md) and [RCS v1 JSON Schema](rcs-v1-schema.md).

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
| Handling | Fix RCS JSON against [v1.schema.json](../contracts/rcs-schema.md) |

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

### RerankError

| Aspect | Value |
|--------|-------|
| Typical cause | Cohere or rerank provider failure |
| HTTP (server) | 502 |
| Python | Provider-related exception |
| TypeScript | Provider error types |
| Handling | Verify `COHERE_API_KEY` and hybrid config |

## Infrastructure errors

### MemoryError

| Aspect | Value |
|--------|-------|
| Typical cause | Qdrant or Postgres memory backend unavailable |
| HTTP (server) | 503 |
| Python | `MemoryOperationError`, `InfrastructureUnavailableError` |
| TypeScript | Memory / infrastructure errors |
| Handling | Check `ARCFLOW_QDRANT_URL`, Postgres, network |

### Timeout

| Aspect | Value |
|--------|-------|
| Typical cause | Workflow or step timeout exceeded |
| HTTP (server) | 408 |
| Python | `WorkflowTimeoutError` |
| TypeScript | Timeout error types |
| Handling | Increase `exec_config` timeouts or optimize steps |

### RateLimited

| Aspect | Value |
|--------|-------|
| Typical cause | Provider 429 or Relay site RPM exceeded |
| HTTP (server) | 429 |
| Python | May surface as provider or execution error |
| TypeScript | Provider rate limit handling |
| Handling | Backoff; raise site `rate_limit_rpm` if legitimate traffic |

## HITL errors

### HumanTimeout

| Aspect | Value |
|--------|-------|
| Typical cause | HITL approval window expired |
| HTTP (server) | 408 |
| Python | HITL-related execution error |
| TypeScript | HITL error types |
| Handling | Re-run or extend `HitlConfig.timeout_seconds` |

### HumanRejected

| Aspect | Value |
|--------|-------|
| Typical cause | Approver returned `approved: false` |
| HTTP (server) | 422 |
| Python | `HumanRejectedError` |
| TypeScript | HITL rejection types |
| Handling | Expected business outcome; branch workflow |

### ApprovalNotFound

| Aspect | Value |
|--------|-------|
| Typical cause | Wrong `approval_key` on approve POST |
| HTTP (server) | 404 |
| Python | HITL error |
| TypeScript | HITL error |
| Handling | Use interrupt payload key from GET run |

### AlreadyApproved

| Aspect | Value |
|--------|-------|
| Typical cause | Duplicate approve POST |
| HTTP (server) | 409 |
| Python | HITL error |
| TypeScript | HITL error |
| Handling | Treat as idempotent success path in client |

## Quick lookup table

| ErrorCode | HTTP | Category |
|-----------|------|----------|
| WorkflowNotFound | 404 | Definition |
| InvalidWorkflowDefinition | 400 | Definition |
| UnsupportedRcsVersion | 400 | Definition |
| StepExecutionFailed | 422/500 | Execution |
| ToolExecutionFailed | 422 | Execution |
| InternalError | 500 | Execution |
| ProviderError | 502 | Provider |
| EmbeddingError | 502 | Provider |
| RerankError | 502 | Provider |
| MemoryError | 503 | Infrastructure |
| Timeout | 408 | Infrastructure |
| RateLimited | 429 | Infrastructure |
| HumanTimeout | 408 | HITL |
| HumanRejected | 422 | HITL |
| ApprovalNotFound | 404 | HITL |
| AlreadyApproved | 409 | HITL |

## Recommended handling patterns

| Category | Client behavior |
|----------|-----------------|
| Definition | Fail fast; fix workflow before retry |
| Provider / Memory | Retry with exponential backoff |
| RateLimited | Honor `retry_after` if present |
| HITL | Poll GET run; present approve UI |
| Timeout | Increase limits or split workflow |

## Trace correlation

Failed runs emit `WorkflowFailed` or `StepFailed` with `error_code` string matching enum names. Use trace export for support without logging user input.

## Related pages

- [RCS schema](rcs-schema.md)
- [Retry and backoff](../guides/reliability/retry-and-backoff.md)
- [HITL overview](../guides/human-in-the-loop/hitl-overview.md)
