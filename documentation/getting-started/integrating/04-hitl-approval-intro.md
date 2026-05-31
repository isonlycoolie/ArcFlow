# 04 HITL approval intro

**Audience:** `[developer]` `[operator]`

## Before you start

Complete [02 Server API first run](02-server-api-first-run.md). HITL (human-in-the-loop) requires `arcflow-server` with Postgres and `recovery_enabled: true` on the run. Embedded SDK runs without the server cannot persist interrupt state across restarts in production.

[03 External callbacks intro](03-external-callbacks-intro.md) also ends in `Interrupted` status, but external callbacks wait for integrator HTTP posts. HITL waits for a human approver on a dedicated approve route.

## Concept

HITL pauses a workflow at a designated step until a human approves or rejects the pending action. ArcFlow treats the pause as a first-class run state (`Interrupted`), not an error. After the gated step completes its agent work, the runtime stores interrupt metadata in Postgres and waits for:

```text
POST /v1/runs/{run_id}/approve/{approval_key}
```

If the approver sends `approved: true`, the engine resumes and runs downstream steps. If `approved: false`, the run fails with error code `HumanRejected`. If `timeout_seconds` elapses with no decision, the run fails with `HumanTimeout`.

Attach HITL per step, not on the workflow root. Each gate declares a stable `approval_key` used in the approve URL.

State machine (HITL branch):

```text
Running
  -> step with hitl completes agent work
       -> Interrupted
            -> approve (approved: true)  -> Running -> Completed
            -> approve (approved: false) -> Failed (HumanRejected)
            -> timeout                   -> Failed (HumanTimeout)
```

Python SDK callers may catch `WorkflowInterruptedError` with `run_id` and `approval_key` when targeting the HTTP server with recovery enabled.

## Example

Three-step expense flow matching `examples/hitl/expense_approval.py`:

```python
import os

os.environ.setdefault("ARCFLOW_SERVER_API_KEY", "dev-secret")

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
```

When interrupted, poll run status:

```bash
curl -s "http://localhost:8080/v1/runs/RUN_ID" \
  -H "Authorization: Bearer dev-secret"
```

The response includes an `interrupt` object with `approval_key` and `expires_at` (metadata only, no full conversation text).

Approve to resume:

```bash
curl -sS -X POST "http://localhost:8080/v1/runs/RUN_ID/approve/manager_approval" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer dev-secret" \
  -d '{"approved": true, "data": {"manager_id": "mgr-42"}}'
```

Or use the helper script:

```bash
bash examples/hitl/approve_cli.sh RUN_ID
```

Poll again until `status` is `Completed` and read `result.output`.

## Verify

| Check | Expected |
|-------|----------|
| Run stops after manager step | `status` is `Interrupted` |
| Accounting step before approve | Does not run |
| Approve with `approved: true` | Run completes; accounting step runs |
| Reject with `approved: false` | `Failed` with `HumanRejected` |
| Wrong `approval_key` | `ApprovalNotFound` (404) |

Trace exports remain SEC-1 metadata only. You see step lifecycle events, not approver notes, in trace storage.

## Next

| Goal | Document |
|------|----------|
| Step-level HITL fields | [Configuring interrupts](../../guides/human-in-the-loop/configuring-interrupts.md) |
| Approve API detail | [Approve and reject](../../guides/human-in-the-loop/approve-and-reject.md) |
| Full tutorial | [Track E: HITL and external](../../tutorials/track-e-hitl-and-external.md) |
| Static browser client | [Static site chatbot](../paths/static-site-chatbot.md) |

When external binding recovery sets `on_fatal: hitl_escalate`, failed external work can route into the same HITL approve flow. See [External callbacks](../../guides/external-integrations/external-callbacks.md).

## Source

`runtime/arcflow-core/src/human/`, `server/arcflow-server/src/handlers/approve.rs`, `examples/hitl/`; [HITL overview](../../guides/human-in-the-loop/hitl-overview.md); capabilities reference §8, Appendix E.
