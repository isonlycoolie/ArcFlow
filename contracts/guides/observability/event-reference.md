# Trace event reference

Events are defined in `runtime/arcflow-core/src/tracing/events.rs` and documented in [trace-events-v1.md](../../normative/observability/trace-events-v1.md). All events carry metadata only — no user input, tool output, or memory values.

## Workflow lifecycle

| Event | When emitted |
|-------|----------------|
| `WorkflowStarted` | Run begins; includes workflow name and step count |
| `WorkflowCompleted` | All steps finished successfully |
| `WorkflowFailed` | Run halted after a step failure |
| `WorkflowValidationFailed` | Workflow definition invalid before execution |

## Step lifecycle

| Event | When emitted |
|-------|----------------|
| `StepStarted` | Step begins; agent name and role (metadata) |
| `StepCompleted` | Step finished; duration, tokens, output size |
| `StepFailed` | Step error; error code and sanitized message |
| `StateCommitted` | State engine committed step output |

## Agent / provider

| Event | When emitted |
|-------|----------------|
| `AgentInvoked` | Agent called; input **size** only |
| `AgentResponseReceived` | Agent returned; output **size** only |
| `TokensConsumed` | Token usage recorded for a step |
| `ProviderRequestSent` | LLM request metadata (Sprint 6+) |
| `ProviderResponseReceived` | LLM response metadata |
| `ProviderRateLimited` | Provider rate limit hit |
| `ProviderError` | Provider failure |

## Tools

| Event | When emitted |
|-------|----------------|
| `ToolCallStarted` | Tool invocation begins; schema hash only |
| `ToolCallCompleted` | Tool succeeded; output **size** only |
| `ToolCallFailed` | Tool failed; error code |
| `ToolInputValidationFailed` | JSON Schema validation failed |

## Memory

| Event | When emitted |
|-------|----------------|
| `MemoryWrite` | Write to session/shared/persistent/vector; key only |
| `MemoryRead` | Read attempt; key and hit/miss |
| `MemoryDegraded` | Backend unavailable; fallback path |
| `MemoryEvicted` | Entry evicted from bounded store |

## Storage / retry

| Event | When emitted |
|-------|----------------|
| `TraceStorageWarning` | Per-run event cap reached; one warning per run |
| `RetryAttempted` | Step retry with backoff (Sprint 7) |
| `RetryExhausted` | All retry attempts failed (Sprint 7) |
| `TimeoutEnforced` | Step or workflow timeout (Sprint 7) |
