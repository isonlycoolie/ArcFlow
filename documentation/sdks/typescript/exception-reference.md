# TypeScript SDK exception reference


TypeScript errors inherit from `ArcFlowError`. Native Rust failures arrive as thrown Error objects with structured message prefixes; `mapNativeError()` converts them into typed subclasses when patterns match.

## Exported exception classes

| Class | Extends | When raised |
|-------|---------|-------------|
| `ArcFlowError` | `Error` | Base; fallback from `mapNativeError` |
| `WorkflowConfigurationError` | `ArcFlowError` | Invalid workflow before or at validation |
| `WorkflowExecutionError` | `ArcFlowError` | Step failure during run |
| `ProviderConfigurationError` | `ArcFlowError` | Invalid provider constructor args |
| `ProviderExecutionError` | `ArcFlowError` | LLM call failed |
| `RetryExhaustedError` | `ArcFlowError` | All retries exhausted |
| `WorkflowTimeoutError` | `ArcFlowError` | Workflow or step timeout |
| `TraceNotFoundError` | `ArcFlowError` | No trace for last run |
| `HumanRejectedError` | `ArcFlowError` | HITL rejection |
| `WorkflowInterruptedError` | `ArcFlowError` | HITL interrupt (expected pause) |

## mapNativeError

```typescript
import { mapNativeError } from "arcflow";

try {
 await wf.run("input");
} catch (err) {
 throw mapNativeError(err);
}
```

Mapping rules in `arcflow/exceptions.ts`:

| Native message pattern | Mapped type |
|------------------------|-------------|
| `ProviderExecutionError\|...` | `ProviderExecutionError` with `providerId`, `runId`, `failedStep` |
| `WorkflowExecutionError\|...` | `WorkflowExecutionError` with `runId`, `failedStep` |
| `failed after N attempts` | `RetryExhaustedError` with `attemptsMade` |
| contains `timed out` | `WorkflowTimeoutError` with `timeoutType` |
| `No trace found` | `TraceNotFoundError` |
| `invalid`, `must be`, `Cannot run` | `WorkflowConfigurationError` |
| otherwise | `ArcFlowError` |

Pipe-separated native messages encode context fields before the human-readable tail.

## Typed fields

### WorkflowExecutionError

| Field | Type |
|-------|------|
| `runId` | `string \| undefined` |
| `failedStep` | `string \| undefined` |

### ProviderExecutionError

| Field | Type |
|-------|------|
| `providerId` | `string \| undefined` |
| `runId` | `string \| undefined` |
| `failedStep` | `string \| undefined` |

### RetryExhaustedError

| Field | Type |
|-------|------|
| `attemptsMade` | `number \| undefined` |

### WorkflowTimeoutError

| Field | Type |
|-------|------|
| `timeoutType` | `string \| undefined` (`workflow` or `step` heuristic) |

### HumanRejectedError

| Field | Type |
|-------|------|
| `approvalKey` | `string \| undefined` |

### WorkflowInterruptedError

| Field | Type |
|-------|------|
| `runId` | `string` |
| `approvalKey` | `string` |
| `expiresAt` | `string \| undefined` |

## Parity gaps vs Python exceptions

These Python exception classes are **not** exported from the TypeScript SDK. Native failures in the same category may surface as generic `ArcFlowError` or `WorkflowExecutionError` until dedicated bindings are added:

| Python only | Typical native symptom |
|-------------|------------------------|
| `ToolConfigurationError` | Pre-run tool validation message |
| `ToolExecutionError` | Tool failure during run |
| `MemoryConfigurationError` | Invalid memory config |
| `MemoryOperationError` | Backend I/O failure |
| `InfrastructureUnavailableError` | Postgres/Qdrant unreachable |
| `TraceStorageWarning` | Trace store capacity drop |

If you need stable `instanceof` checks for these cases in TypeScript, inspect `err.message` prefixes or use Python SDK for tool/memory-heavy workflows.

## Configuration vs execution

Same discipline as Python:

| Phase | Types |
|-------|-------|
| Pre-run (fix definition) | `WorkflowConfigurationError`, `ProviderConfigurationError` |
| During run | `WorkflowExecutionError`, `ProviderExecutionError`, `RetryExhaustedError`, `WorkflowTimeoutError` |
| HITL control flow | `WorkflowInterruptedError`, `HumanRejectedError` |
| Post-run | `TraceNotFoundError` |

## Remediation quick reference

| Exception | Typical fix |
|-----------|-------------|
| `WorkflowConfigurationError` | Fix workflow graph/steps, call order, empty input |
| `WorkflowExecutionError` | `wf.trace()`, inspect failed step |
| `ProviderConfigurationError` | Valid model name, temperature, maxTokens |
| `ProviderExecutionError` | API key env var, model availability |
| `RetryExhaustedError` | Increase `withRetry` or fix root error |
| `WorkflowTimeoutError` | `withTimeout` / `withStepTimeout` |
| `TraceNotFoundError` | Call `run()` before `trace()` |
| `WorkflowInterruptedError` | `resumeWithApproval()` |
| `HumanRejectedError` | Handle rejection branch |

Messages use the `[ArcFlow]` prefix when raised from validation layers, matching Python convention.
