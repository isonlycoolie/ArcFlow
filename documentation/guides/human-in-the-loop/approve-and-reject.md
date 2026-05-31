**Audience:** `[developer]` `[operator]`

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

The engine records approval, resumes from the interrupted step, runs remaining steps, and marks the run `Completed` when finished. The `data` object is stored for audit in run state; it does not appear in SEC-1 trace events.

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
