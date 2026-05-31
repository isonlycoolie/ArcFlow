**Audience:** `[developer]` `[platform]`

# Timeouts

Timeouts cap how long a workflow or individual step may run before the engine terminates with `TimeoutEnforced` traces and `Timeout` error code (HTTP 408 on server). Human-in-the-loop steps use a separate approval window via `HitlConfig.timeout_seconds`.

Control plane: [Execution model](../../concepts/execution-model.md). Works alongside [Retry and backoff](retry-and-backoff.md) (retries consume time within the same timeout budget per engine rules).

## Workflow timeout

Set on `exec_config`:

```json
{
  "exec_config": {
    "workflow_timeout_secs": 600,
    "step_timeout_secs": 120
  }
}
```

| Field | Scope |
|-------|-------|
| `workflow_timeout_secs` | Entire run from start to terminal status |
| `step_timeout_secs` | Each step execution ceiling |

Example server run:

```json
{
  "workflow": { "name": "demo", "execution_mode": "linear", "steps": [] },
  "agents": [],
  "input": "Long running task",
  "exec_config": {
    "recovery_enabled": true,
    "workflow_timeout_secs": 300,
    "step_timeout_secs": 60
  }
}
```

## Trace event: TimeoutEnforced

### Workflow timeout

```json
{
  "kind": "TimeoutEnforced",
  "run_id": "r1",
  "step_id": null,
  "timeout_type": "workflow",
  "configured_ms": 600000,
  "elapsed_ms": 600012
}
```

### Step timeout

```json
{
  "kind": "TimeoutEnforced",
  "run_id": "r1",
  "step_id": "s1",
  "timeout_type": "step",
  "configured_ms": 120000,
  "elapsed_ms": 120005
}
```

Terminal status: `Failed` with `error_code: "Timeout"`.

## HITL timeout

Attach to step:

```json
{
  "id": "s-approve",
  "agent_id": "a-review",
  "order": 2,
  "hitl": {
    "approval_key": "manager_signoff",
    "timeout_seconds": 86400
  }
}
```

Requires `exec_config.recovery_enabled: true` so interrupt state persists in Postgres.

When approver does not act within `timeout_seconds`:

- Run fails with `HumanTimeout` error code (HTTP 408)
- Distinct from workflow/step execution timeouts

Approve flow (no timeout):
