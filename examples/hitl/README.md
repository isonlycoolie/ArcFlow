# Expense Reimbursement with Manager Approval

## Problem

**Northwind Consulting** employees submit expense reports in Slack. Finance policy requires:

1. Employee submits amount + receipt description
2. **Manager must approve** any reimbursement over the team threshold
3. Accounting posts to the ledger only after approval

Manual approval threads get lost; the team wants a workflow that **pauses** until a manager acts.

## Who this is for

| Role | Goal |
|------|------|
| **Finance ops** | Enforce approval before posting |
| **Engineering** | HITL interrupt + Postgres recovery + resume |

## What ArcFlow demonstrates

- `HitlConfig(approval_key=manager_approval)` on manager step
- `enable_recovery()` for interrupt persistence
- `WorkflowInterruptedError` handling with approve CLI hint
- Optional server runtime via `ARCFLOW_RUNTIME`

## Prerequisites

```bash
# Install the SDK for development (editable)
pip install -e sdk-python

# Or install the published SDK from PyPI for normal use:
pip install arcflow-sdk

docker compose -f docker/docker-compose.server.yml up -d
export ARCFLOW_RUNTIME=http://localhost:8080
```

## Run

```bash
python examples/hitl/expense_approval.py
```

When interrupted, approve:

```bash
bash examples/hitl/approve_cli.sh <run_id>
```

## Verify

- First run prints `Interrupted run_id=... approval_key=manager_approval`
- After approve: workflow completes accounting step
- Server `GET /v1/runs/{id}` shows `Interrupted`, then `Running`, then `Completed`

## Production notes

- Map `approval_key` to your IAM roles in the approving service
- Set `timeout_seconds` per finance SLA
- Audit approve payloads in Postgres (`arcflow_human_approvals`)

## Files

| File | Purpose |
|------|---------|
| [`expense_approval.py`](expense_approval.py) | Three-step workflow with HITL |
| [`approve_cli.sh`](approve_cli.sh) | curl helper for approve endpoint |
