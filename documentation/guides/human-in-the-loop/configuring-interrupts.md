**Audience:** `[developer]`

# Configuring HITL interrupts

HITL attaches to individual steps, not to the workflow root. Each gated step declares an `approval_key` that approvers use in the approve URL, plus an optional timeout. This guide covers RCS fields, server interrupt payloads, and a support workflow pattern you can copy.

## Step definition fields

| Field | Required | Purpose |
|-------|----------|---------|
| `approval_key` | yes | Stable string passed in `POST .../approve/{approval_key}` |
| `timeout_seconds` | no (default 3600 in SDK) | Seconds until `HumanTimeout` if no decision |
| `interrupt` | no (default true) | When false, step skips the gate (rare; used in tests) |

RCS example on a step:

```json
{
  "id": "00000000-0000-4000-8000-000000000030",
  "agent_id": "00000000-0000-4000-8000-000000000040",
  "order": 2,
  "hitl": {
    "approval_key": "manager_signoff",
    "timeout_seconds": 86400
  }
}
```

`exec_config` for the run must include `"recovery_enabled": true`. Python:

```python
wf = Workflow("expense_reimbursement", runtime=RUNTIME).enable_recovery()
```

## Python support workflow example

This pattern matches `examples/hitl/expense_approval.py`: submit, manager gate, accounting post.

```python
import os
from arcflow import Agent, HitlConfig, Workflow
from arcflow.hitl import WorkflowInterruptedError

RUNTIME = os.environ.get("ARCFLOW_RUNTIME", "http://localhost:8080")
APPROVAL_KEY = "manager_approval"

submit = Agent(name="submit", role="employee", instructions="Submit expense request.")
manager = Agent(name="manager", role="reviewer", instructions="Manager review gate.")
accounting = Agent(name="accounting", role="finance", instructions="Post to accounting.")

wf = (
    Workflow("expense_reimbursement", runtime=RUNTIME)
    .enable_recovery()
    .step(submit)
    .step(manager, hitl=HitlConfig(approval_key=APPROVAL_KEY, timeout_seconds=3600))
    .step(accounting)
)

try:
    result = wf.run("amount=250.00;desc=client lunch")
    print(result.run_id, result.status)
except WorkflowInterruptedError as exc:
    print(f"Interrupted run_id={exc.run_id} approval_key={exc.approval_key}")
    print(f"Approve: POST /v1/runs/{exc.run_id}/approve/{exc.approval_key}")
```

When the manager step finishes agent work, the run stops at `Interrupted`. Downstream accounting does not run until approval.

## Interrupt payload on GET /v1/runs/{id}

Poll run status while waiting for a human decision:

```bash
curl -s "http://localhost:8080/v1/runs/RUN_ID" \
  -H "Authorization: Bearer ${ARCFLOW_SERVER_API_KEY}"
```

When status is `Interrupted`, the response includes an `interrupt` object (metadata only):

```json
{
  "run_id": "550e8400-e29b-41d4-a716-446655440000",
  "trace_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
  "status": "Interrupted",
  "interrupt": {
    "approval_key": "manager_approval",
    "expires_at": "2026-06-01T12:00:00Z",
    "step_index": 1
  },
  "created_at": "2026-05-31T10:00:00Z"
}
```

Build approver UIs from `approval_key`, `expires_at`, and `step_index`. Fetch business context from your own ticket store; do not expect full LLM transcripts in this payload.

