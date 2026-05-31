**Audience:** `[platform]` `[developer]`

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
