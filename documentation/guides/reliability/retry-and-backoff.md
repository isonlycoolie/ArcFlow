**Audience:** `[developer]`

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
