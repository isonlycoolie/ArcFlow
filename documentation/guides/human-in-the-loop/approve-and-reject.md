
# Approve and reject HITL runs

Once a run reaches `Interrupted`, approvers call a single HTTP endpoint to resume or reject. This guide covers request bodies, outcomes, timeout behavior, error handling, and Relay proxy considerations.

## Approve endpoint

```text
POST /v1/runs/{run_id}/approve/{approval_key}
Authorization: Bearer <ARCFLOW_SERVER_API_KEY>
Content-Type: application/json
```

The path `approval_key` must match the key on the interrupted step and the `interrupt.approval_key` field from `GET /v1/runs/{run_id}`.

### Approve (resume)

```json
{
 "approved": true,
 "data": {
 "manager_id": "mgr-42",
 "notes": "Looks good"
 }
}
```

The engine records approval, resumes from the interrupted step, runs remaining steps, and marks the run `Completed` when finished. The `data` object is stored for audit in run state; it does not appear in trace data policy trace events.

Example with curl:

```bash
curl -sS -X POST "http://localhost:8080/v1/runs/${RUN_ID}/approve/manager_approval" \
 -H "Content-Type: application/json" \
 -H "Authorization: Bearer ${ARCFLOW_SERVER_API_KEY}" \
 -d '{"approved": true, "data": {"manager_id": "mgr-42"}}'
```

Success response shape:

```json
{
 "status": "Completed",
 "message": "Approval recorded, workflow completed"
}
```

Poll `GET /v1/runs/{run_id}` to read final `result.output` when status is `Completed`.

### Reject (terminal failure)

```json
{
 "approved": false,
 "data": {
 "reason": "Insufficient evidence"
 }
}
```

The run moves to `Failed` with error code `HumanRejected`. Downstream steps do not execute. SDK callers may see `HumanRejectedError` if they resume in-process after a reject path.

## Timeout behavior

Each `HitlConfig` sets `timeout_seconds`. If no approve call arrives before expiry, the next approve attempt receives `HumanTimeout` (HTTP 410 Gone on the server). The run remains failed; create a new run for a fresh approval cycle.

Design operator alerts around `interrupt.expires_at` from the run status payload so approvers act before timeout.

## Error handling

| Error code | Typical HTTP | Cause | Operator action |
|------------|--------------|-------|-----------------|
| `ApprovalNotFound` | 404 | Wrong key or run not interrupted | Confirm run_id and key from interrupt payload |
| `AlreadyApproved` | 409 | Second approve on same gate | Treat as idempotent success if run already completed |
| `HumanTimeout` | 410 | Past `expires_at` | Start a new run or escalate offline |
| `HumanRejected` | (terminal state) | `approved: false` | Close ticket; do not retry same run |

Non-interrupted runs return 400 on approve:

```text
[ArcFlow] run '...' is not awaiting approval (status=Completed)
```

## Relay proxy

Static product browsers must not hold server admin keys. When site policy and scoped runtime keys allow, Relay can proxy approve calls to upstream `arcflow-server` on behalf of an authenticated operator session. Confirm your deployment's Relay route map and key scopes before exposing approve in the browser.

Direct server approve from backend services uses the standard Bearer runtime key.

## End-to-end operator script

`examples/hitl/approve_cli.sh` approves the expense demo:

```bash
export ARCFLOW_SERVER_API_KEY=dev-secret
./examples/hitl/approve_cli.sh RUN_ID
```

Replace `RUN_ID` with the UUID printed when `expense_approval.py` raises `WorkflowInterruptedError`.

## Trace visibility (trace data policy)

Traces show lifecycle metadata around the gate, not approval notes:

```json
[
 { "kind": "StepCompleted", "run_id": "r1", "step_id": "s1", "step_index": 0, "duration_ms": 400, "output_size_bytes": 120 },
 { "kind": "StepStarted", "run_id": "r1", "step_id": "s2", "step_index": 1, "agent_name": "manager", "agent_role": "reviewer" },
 { "kind": "StepCompleted", "run_id": "r1", "step_id": "s2", "step_index": 1, "duration_ms": 300, "output_size_bytes": 80 }
]
```

After approval and resume, expect further `StepStarted` / `StepCompleted` events for downstream steps. No field carries `data.notes` from the approve body.

## Related pages

- [Configuring interrupts](configuring-interrupts.md) for `approval_key` and timeout setup
- [HITL overview](hitl-overview.md) for the state machine
- [Trace data policy rules](../observability/sec-1-rules.md) for trace field policy
