
# Retry and backoff

ArcFlow retries failed steps and workflows using configurable policies at workflow level (`exec_config.retry`) and step level (`retry_policy` on steps or workflow). Backoff kinds include fixed, linear, and exponential with optional jitter. Traces emit `RetryAttempted` and `RetryExhausted` for observability.

Reliability context: [Execution model](../../concepts/execution-model.md). Complements [Step fallbacks](../workflows/step-fallbacks.md) (alternate agent after failure) and [Timeouts](timeouts.md) (hard ceilings).

## Workflow-level retry (exec_config)

```json
{
 "exec_config": {
 "recovery_enabled": true,
 "retry": {
 "max_attempts": 3,
 "backoff": {
 "kind": "exponential",
 "base_ms": 1000,
 "multiplier": 2.0,
 "max_ms": 30000,
 "jitter_ms": 100
 }
 }
 }
}
```

Parsed in `server/arcflow-server/src/exec_config.rs`; same shape for SDK runs.

## Backoff kinds

| kind | Behavior |
|------|----------|
| `fixed` | Constant delay between attempts |
| `linear` | Delay increases linearly with attempt number |
| `exponential` | Delay multiplied by `multiplier` each attempt, capped at `max_ms` |

### Fixed example

```json
{
 "backoff": {
 "kind": "fixed",
 "base_ms": 2000,
 "jitter_ms": 0
 }
}
```

### Linear example

```json
{
 "backoff": {
 "kind": "linear",
 "base_ms": 1000,
 "multiplier": 1.5,
 "max_ms": 15000,
 "jitter_ms": 50
 }
}
```

### Exponential example (recommended for provider flakiness)

```json
{
 "backoff": {
 "kind": "exponential",
 "base_ms": 1000,
 "multiplier": 2.0,
 "max_ms": 30000,
 "jitter_ms": 100
 }
}
```

`jitter_ms` spreads retries to avoid thundering herds against rate-limited providers.

## Step-level retry_policy

Workflow definition may set per-step overrides:

```json
{
 "steps": [
 {
 "id": "s1",
 "agent_id": "a1",
 "order": 1,
 "retry_policy": {
 "max_attempts": 5,
 "backoff": {
 "kind": "exponential",
 "base_ms": 500,
 "multiplier": 2.0,
 "max_ms": 10000,
 "jitter_ms": 25
 }
 }
 }
 ],
 "retry_policy": {
 "max_attempts": 2,
 "backoff": { "kind": "fixed", "base_ms": 1000 }
 }
}
```

Step policy applies to that step; workflow policy applies as default where step policy is absent.

## Trace events

### RetryAttempted

```json
{
 "kind": "RetryAttempted",
 "run_id": "r1",
 "step_id": "s1",
 "attempt_number": 2,
 "max_attempts": 3,
 "backoff_ms": 2000,
 "trigger_error_code": "ProviderError"
}
```

### RetryExhausted

```json
{
 "kind": "RetryExhausted",
 "run_id": "r1",
 "step_id": "s1",
 "total_attempts": 3,
 "last_error_code": "ProviderError"
}
```

After exhaustion, the step fails unless [Step fallbacks](../workflows/step-fallbacks.md) routes to a fallback step (`StepFallbackActivated`).

## Provider rate limits

`ProviderRateLimited` traces may include `retry_after_seconds`. Combine retry policy with provider guidance; respect 429 semantics from [Provider configuration](../agents-and-tools/provider-configuration.md).

Terminal mapping: `RateLimited` error code (HTTP 429) when retries do not recover.

## Test mode without live LLM

```json
{
 "exec_config": {
 "recovery_enabled": false,
 "retry": {
 "max_attempts": 3,
 "backoff": { "kind": "fixed", "base_ms": 10 }
 },
 "test": {
 "steps": {
 "s1": {
 "fail_times": 2,
 "output": "fail",
 "then_output": "success on third try"
 }
 }
 }
 }
}
```

Expect two `RetryAttempted` events then `StepCompleted`. See [Validation and testing](../workflows/validation-and-testing.md).

## External bindings

HTTP callbacks use separate recovery on bindings:

```json
{
 "external_bindings": [
 {
 "id": "payment_webhook",
 "kind": "http_callback",
 "attach_to_step_id": "s-pay",
 "mode": "async",
 "recovery": {
 "max_attempts": 3,
 "on_failure": "retry_with_backoff"
 }
 }
 ]
}
```

Traces: `ExternalRecoveryTriggered`. Distinct from step retry but same backoff concepts.

## Run status during retry

`ExecutionStatus` may show `Retrying` between attempts, then return to `Running`. Poll `GET /v1/runs/{id}` on server integrations.

## Related pages

- [Step fallbacks](../workflows/step-fallbacks.md)
- [Timeouts](timeouts.md)
- [Recovery and resume](recovery-and-resume.md)
- [Server API quickstart](../../getting-started/quickstart-server-api.md)
