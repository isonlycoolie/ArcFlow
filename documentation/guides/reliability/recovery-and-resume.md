
# Recovery and resume

Recovery persists run and step progress to PostgreSQL when `exec_config.recovery_enabled` is true. After failure or interrupt, a new run or resume API can continue from the last committed state instead of restarting from step zero. Linear recovery is production-ready. Graph checkpoint persistence exists, but full resume dispatch from mid-graph checkpoints is **partial (FP-1.01)**.

Architecture: [Execution model](../../concepts/execution-model.md). Known gap detail: [Maturity and known gaps](../../concepts/maturity-and-known-gaps.md).

## Enabling recovery

```json
{
  "exec_config": {
    "recovery_enabled": true,
    "workflow_timeout_secs": 600,
    "step_timeout_secs": 120
  }
}
```

Requirements:

- `ARCFLOW_POSTGRESQL_URL` on server (or SDK with Postgres-backed features)
- Migrations applied: `arcflow migrate up` (000001 through 000007)
- Server `/ready` returns 200 only when migrations are current

Embedded SDK runs without Postgres skip persistence unless you configure a database URL for recovery features.

## What gets persisted

| Store | Contents |
|-------|----------|
| `arcflow_recovery_state` | Recovery id, completed steps, graph checkpoint fields |
| `arcflow_runs` | Run status, result snapshot, interrupt payload |

### RecoveryState fields (Postgres)

| Field | Linear | Graph |
|-------|--------|-------|
| `recovery_id` | Yes | Yes |
| `original_run_id` | Yes | Yes |
| `completed_steps` | Step indices committed | Node completion tracking |
| `failed_at_step_index` | Resume point hint | May apply |
| `execution_mode` | `linear` | `graph` |
| `current_node_id` | N/A | Last checkpoint node |
| `graph_iteration_count` | N/A | Loop counter |
| `pending_join` | N/A | Join synchronization state |

Graph columns added in migration `20240531000002`.

## Linear resume (production-ready)

After each committed step, state is durable. On resume:

1. `WorkflowRecoveryStarted` trace with `resume_from_step`
2. Engine skips completed steps
3. Continues from next step
4. `WorkflowRecoveryCompleted` with `steps_re_executed`

Example trace excerpt:

```json
[
  {
    "kind": "WorkflowRecoveryStarted",
    "run_id": "r2",
    "original_run_id": "r1",
    "resume_from_step": 1
  },
  { "kind": "StepStarted", "run_id": "r2", "step_id": "s2", "step_index": 1 },
  {
    "kind": "WorkflowRecoveryCompleted",
    "run_id": "r2",
    "original_run_id": "r1",
    "steps_re_executed": 1
  }
]
```

Linear flows: [Linear workflows](../workflows/linear-workflows.md).

## Graph checkpoints (FP-1.01 partial)

After each graph node, when recovery is enabled, `persist_graph_checkpoint` upserts:

- `current_node_id`
- `graph_iteration_count`
- `pending_join` (join synchronization)

**FP-1.01:** Schema and persist path are implemented. Resume dispatch that continues mid-DAG from checkpoint is **incomplete**. Do not depend on graph resume in production until FP-1.01 closes (plan ref `feat/fp-1-graph-recovery`).

Safe today:

- Graph forward execution with checkpoints written
- Linear recovery
- Re-run graph workflows from entry after failure (new run)

Not production-complete:

- Resume API continuing exact graph node after crash mid-parallel branch

Graph authoring: [Graph workflows](../workflows/graph-workflows.md).

## HITL and recovery

Human-in-the-loop **requires** `recovery_enabled`. Interrupt state lives in Postgres (`arcflow_human_approvals`, run snapshot columns, migration `20240531000004`).

Flow:

```text
Running → HITL step → Interrupted
POST approve(approved=true) → Running → Completed
POST approve(approved=false) → Failed (HumanRejected)
Timeout → Failed (HumanTimeout)
```

Poll interrupt payload:

```json
GET /v1/runs/{run_id}

{
  "run_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
  "status": "Interrupted",
  "interrupt": {
    "approval_key": "manager_signoff",
    "step_id": "00000000-0000-4000-8000-000000000030",
    "metadata": { "summary_bytes": 256 }
  }
}
```

Approve:

```json
POST /v1/runs/{run_id}/approve/manager_signoff

{ "approved": true, "data": { "notes": "Looks good" } }
```

Response:

```json
{ "run_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7", "status": "Running" }
```

## Idempotency

`Idempotency-Key` header on `POST /v1/runs` deduplicates identical submissions within the server window (Postgres-backed). Distinct from recovery resume but prevents duplicate runs from client retries.

## Migrations and operations

```bash
# Before deploy
arcflow migrate up

# Smoke
curl -s http://localhost:8080/health
curl -s http://localhost:8080/ready
```

Pending migrations cause `/ready` to fail. See [Install and build](../../getting-started/install-and-build.md) and [Server API quickstart](../../getting-started/quickstart-server-api.md).

## Run state machine

```text
Pending → Running → Completed
                 └→ Failed → (recovery?) → WorkflowRecoveryStarted → Running
                 └→ Interrupted (HITL) → approve → Running
                 └→ Cancelled
```

`ExecutionStatus` values: `Pending`, `Running`, `Completed`, `Failed`, `Retrying`, `Cancelled`, `Interrupted`.

## Trace persistence

When configured, SEC-1 events also land in `arcflow_trace_events` (migration `20240531000005`). CLI: `arcflow trace <run_id>`. Server: `GET /v1/runs/{id}/trace`.

## Testing recovery

Use test mode for step outputs plus recovery enabled:

```json
{
  "exec_config": {
    "recovery_enabled": true,
    "test": {
      "steps": {
        "s1": { "output": "done" },
        "s2": { "output": "done" }
      }
    }
  }
}
```

Simulate failure between steps in integration tests by aborting process mid-run, then invoking resume API per your deployment's supported surface (server/SDK).

## Related pages

- [Retry and backoff](retry-and-backoff.md)
- [Timeouts](timeouts.md)
- [Validation and testing](../workflows/validation-and-testing.md)
- [Architecture overview](../../concepts/architecture-overview.md)
