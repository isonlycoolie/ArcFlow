
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

```json
POST /v1/runs/{run_id}/approve/manager_signoff

{ "approved": true, "data": { "notes": "Approved" } }
```

Reject:

```json
{ "approved": false, "data": { "reason": "Insufficient evidence" } }
```

Yields `HumanRejected` (HTTP 422).

## Choosing limits

| Workload | Suggested starting point |
|----------|-------------------------|
| Chat single agent | `step_timeout_secs`: 60-120 |
| Multi-step research | `step_timeout_secs`: 180, `workflow_timeout_secs`: 900 |
| HITL business approval | `timeout_seconds`: 86400 (24h) or policy-driven |
| Graph parallel branches | Workflow timeout covers longest parallel path |

Graph runs: [Graph workflows](../workflows/graph-workflows.md). Join nodes wait for branches; workflow timeout applies to wall-clock elapsed time.

## Interaction with retry

Retries extend step duration. If `fail_times` retries each approach `step_timeout_secs`, you may hit step timeout before `RetryExhausted`. Tune both in [Validation and testing](../workflows/validation-and-testing.md):

```json
{
  "exec_config": {
    "step_timeout_secs": 180,
    "retry": { "max_attempts": 3, "backoff": { "kind": "fixed", "base_ms": 5000 } },
    "test": {
      "steps": {
        "s1": { "fail_times": 2, "then_output": "ok" }
      }
    }
  }
}
```

## Provider timeouts

Provider HTTP clients may fail with `ProviderError` before ArcFlow step timeout fires. Traces show `ProviderError` vs `TimeoutEnforced` separately.

## SDK usage

Python:

```python
result = workflow.run(
    "input",
    workflow_timeout_secs=600,
    step_timeout_secs=120,
)
```

TypeScript equivalent on `workflow.run()` options object. See [Python quickstart](../../getting-started/quickstart-python.md) and [TypeScript quickstart](../../getting-started/quickstart-typescript.md).

## Cancellation vs timeout

`ExecutionStatus::Cancelled` is explicit abort. Timeout is engine-enforced failure with `Timeout` code. Recovery may persist state for both depending on when interruption occurred; see [Recovery and resume](recovery-and-resume.md).

## Related pages

- [Retry and backoff](retry-and-backoff.md)
- [Recovery and resume](recovery-and-resume.md)
- [Linear workflows](../workflows/linear-workflows.md)
- [The RCS contract](../../concepts/the-rcs-contract.md) (HitlConfig)
