**Audience:** `[platform]` `[developer]`

# Run state machine

HTTP API run status uses PascalCase `ExecutionStatus` values stored in `arcflow_runs.status`. Polling `GET /v1/runs/{run_id}` on a terminal run always returns the same final state and result snapshot.

## Status values

| Status | Meaning |
|--------|---------|
| `Pending` | Row inserted; execution not yet marked running (brief) |
| `Running` | Engine executing steps |
| `Retrying` | Step failed; retry backoff in progress |
| `Interrupted` | HITL pause; awaiting approve/reject |
| `Completed` | Success; `result` populated |
| `Failed` | Terminal error or human rejection / timeout |
| `Cancelled` | Run cancelled (when supported by caller) |

SDK clients may expose lowercase (`completed`); HTTP uses PascalCase. They refer to the same states.

## Linear run flow

```text
POST /v1/runs
  → Pending → Running → Completed
                    └→ Failed → (retry?) → Retrying → Running
                    └→ Interrupted (HITL) → (approve) → Running
                    └→ Cancelled
```

After create, the server marks the run `Running` and executes synchronously in the request handler for typical stub/short runs, then persists terminal status before returning from background work depending on workflow length. Clients should poll until a terminal status appears.

## HITL detail

A step with `hitl` configuration pauses the workflow:

```text
Running → step with hitl → Interrupted
Interrupted + POST approve (approved=true)  → Running
Interrupted + POST approve (approved=false) → Failed (HumanRejected)
Interrupted + timeout                     → Failed (HumanTimeout)
```

Poll interrupted runs:

```bash
curl -s http://localhost:8080/v1/runs/RUN_ID \
  -H "Authorization: Bearer dev-secret"
```

Example **200** when waiting for approval:

```json
{
  "run_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
  "status": "Interrupted",
  "result": null,
  "error": null,
  "interrupt": {
    "approval_key": "manager_signoff",
    "step_id": "00000000-0000-4000-8000-000000000030",
    "metadata": { "summary_bytes": 256 }
  }
}
```

Approve:

```bash
curl -s -X POST "http://localhost:8080/v1/runs/RUN_ID/approve/manager_signoff" \
  -H "Authorization: Bearer dev-secret" \
  -H "Content-Type: application/json" \
  -d '{"approved": true, "data": {}}'
```

HITL requires `exec_config.recovery_enabled: true` so interrupt state survives process restarts.

## Recovery

When `recovery_enabled` is true and a linear run fails mid-workflow, recovery state is written to `arcflow_recovery_state`. Resume via SDK `workflow.resume(run_id)` or server-side recovery APIs where implemented.

```text
Failed (recovery_enabled) + resume → WorkflowRecoveryStarted → Running → ...
```

**Graph checkpoint resume is partial (FP-1.01).** Graph runs persist `current_node_id`, `graph_iteration_count`, and `pending_join` for observability, but dispatch to continue from checkpoint after crash is incomplete. Treat graph runs as non-resumable for SLA planning until FP-1.01 closes. Linear recovery is production-ready.

## RecoveryState fields (Postgres)

| Field | Role |
|-------|------|
| `recovery_id` | Primary key |
| `original_run_id` | Run being recovered |
| `completed_steps` | Steps finished before failure |
| `failed_at_step_index` | Linear resume pointer |
| `execution_mode` | `linear` or `graph` |
| `current_node_id` | Graph checkpoint (FP-1.01) |
| `graph_iteration_count` | Graph guard counter |
| `pending_join` | Join node state JSON |

## Idempotent polling behavior

Once `status` is `Completed`, `Failed`, or `Cancelled`, repeated `GET /v1/runs/{run_id}` returns the same `result_json` or `error_json` snapshot. Safe for client retry loops and cache-friendly integrations.

## Trace alignment

Trace events follow the same lifecycle: `WorkflowStarted`, step events, optional `RetryAttempted`, `WorkflowInterrupted`, `WorkflowCompleted` or `WorkflowFailed`. Fetch trace:

```bash
curl -s "http://localhost:8080/v1/runs/RUN_ID/trace" \
  -H "Authorization: Bearer dev-secret"
```

## Related pages

- [idempotency.md](idempotency.md) for safe create retries
- [guides/human-in-the-loop/hitl-overview.md](../guides/human-in-the-loop/hitl-overview.md)
- [guides/reliability/recovery-and-resume.md](../guides/reliability/recovery-and-resume.md)
- [postgres-schema.md](postgres-schema.md)

**Source:** capabilities reference Appendix F; `arcflow_core::rcs::types::ExecutionStatus`; `server/arcflow-server/src/handlers/runs.rs`, `server/arcflow-server/src/handlers/approve.rs`.
