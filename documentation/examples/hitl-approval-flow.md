# HITL approval flow example

**Audience:** `[developer]` `[compliance]`

This walkthrough runs a workflow that interrupts for human approval, resumes via the HTTP approve API, and completes the remaining steps. Primary script: [`examples/hitl/expense_approval.py`](../../examples/hitl/expense_approval.py). Approval helper: [`examples/hitl/approve_cli.sh`](../../examples/hitl/approve_cli.sh).

## What this example demonstrates

Human-in-the-loop (HITL) gates a step until an operator approves or rejects through the server API. The expense reimbursement flow submits a request, pauses at manager review, then posts to accounting after approval. State transitions run `Running` to `Interrupted` to `Completed` when approved.

## Prerequisites

| Item | Required |
|------|----------|
| Docker server stack | `docker compose -f docker/docker-compose.server.yml up -d` |
| Postgres | Included in server compose |
| Env | `ARCFLOW_SERVER_API_KEY=dev-secret` (compose default) |
| Python SDK | Built with server runtime support |
| Reading | [HITL overview](../guides/human-in-the-loop/hitl-overview.md) |
| Tutorial track | [Track E](../tutorials/track-e-hitl-and-external.md) |

## Step 1: Start the server

```bash
docker compose -f docker/docker-compose.server.yml up -d --build
curl -sf http://localhost:8080/ready
```

Readiness must return HTTP 200 before running the workflow.

## Step 2: Run the expense workflow

```bash
export ARCFLOW_RUNTIME=http://localhost:8080
export ARCFLOW_SERVER_API_KEY=dev-secret
python examples/hitl/expense_approval.py
```

The script enables recovery and attaches HITL to the manager step:

```python
wf = (
    Workflow("expense_reimbursement", runtime=RUNTIME)
    .enable_recovery()
    .step(submit)
    .step(manager, hitl=HitlConfig(approval_key=APPROVAL_KEY, timeout_seconds=3600))
    .step(accounting)
)
```

On interrupt, it prints the run id and exits zero with approval instructions.

## Step 3: Approve via API

Copy `run_id` from interrupt output, then:

```bash
export ARCFLOW_SERVER_API_KEY=dev-secret
bash examples/hitl/approve_cli.sh YOUR_RUN_ID
```

The script posts to `POST /v1/runs/{run_id}/approve/manager_approval` with `{"approved": true, ...}` then polls run status.

## Step 4: Re-run or poll to completion

After approval, either poll:

```bash
curl -s "http://localhost:8080/v1/runs/YOUR_RUN_ID" \
  -H "X-ArcFlow-Api-Key: dev-secret"
```

Or re-invoke the workflow logic that resumes from checkpoint (server handles continuation after approve). Terminal HTTP status should be `Completed`.

## Expected output

First run (interrupted):

```
Interrupted run_id=<uuid> approval_key=manager_approval
Approve with: examples/hitl/approve_cli.sh <uuid>
```

After approval and completion:

```
Completed run_id=<uuid> output=<truncated accounting summary>
```

HTTP API uses PascalCase statuses (`Interrupted`, `Completed`). SDK embedded runs use lowercase when queried in-process.

## Trace events you should see

| Event kind | When |
|------------|------|
