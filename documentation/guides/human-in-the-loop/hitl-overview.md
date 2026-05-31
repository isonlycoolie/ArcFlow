**Audience:** `[developer]` `[compliance]`

# Human-in-the-loop overview

Human-in-the-loop (HITL) pauses a workflow at a designated step until a human approver or rejects the pending action. ArcFlow treats the pause as a first-class run state (`Interrupted`), not an error. The engine persists enough state to resume after approval, reject to a terminal failure, or fail on timeout.

This model fits expense approvals, clinical disclaimers, trade execution gates, and any step where automated output must not proceed without explicit human consent.

## Interrupt and resume model

A linear or graph workflow runs steps in order until a step with `hitl` configuration completes its agent work. At that gate the runtime:

1. Sets run status to `Interrupted`.
2. Stores interrupt metadata in Postgres (when recovery is enabled).
3. Waits for `POST /v1/runs/{run_id}/approve/{approval_key}`.

If the approver sends `approved: true`, the engine resumes from the interrupted step and continues downstream steps. If the approver sends `approved: false`, the run ends in a failed terminal state with error code `HumanRejected`. If `timeout_seconds` elapses with no decision, the run fails with `HumanTimeout`.

In-process SDK runs raise `WorkflowInterruptedError` with `run_id` and `approval_key` when the workflow targets the HTTP server with recovery enabled. Server-backed runs return `Interrupted` on `GET /v1/runs/{id}` with an `interrupt` object instead of a final result.

## Why recovery is required

HITL requires `exec_config.recovery_enabled: true` (Python: `Workflow.enable_recovery()`). Interrupt state must survive process restarts and multi-replica server deployments. Without Postgres-backed recovery, the engine cannot safely park a run mid-workflow and resume later.

Embedded SDK runs without the server can still declare HITL on steps, but production approval flows almost always go through `arcflow-server` plus Postgres.

## Run state machine (HITL branch)

```text
Running
  └→ step with hitl completes agent work
       └→ Interrupted
            ├→ approve (approved: true)  → Running → Completed
            ├→ approve (approved: false) → Failed (HumanRejected)
            └→ timeout                   → Failed (HumanTimeout)
```

Trace events during the gate follow SEC-1: you see step lifecycle metadata (`StepCompleted` on the gated step before interrupt, later `StepStarted` on resume) but not approval notes or full agent transcripts in trace storage.

## HITL error codes

| Code | When |
|------|------|
| `ApprovalNotFound` | Wrong `approval_key`, or run is not in `Interrupted` |
| `AlreadyApproved` | Duplicate approve on the same key |
| `HumanTimeout` | No decision before `timeout_seconds` |
| `HumanRejected` | Approver sent `approved: false` |

Server handlers map these to HTTP status codes (for example `404` for `ApprovalNotFound`, `409` for `AlreadyApproved`, `410 Gone` for `HumanTimeout`). See [approve and reject](approve-and-reject.md) for request bodies and Relay notes.

## SDK configuration surface

Python attaches HITL per step with `HitlConfig`:

```python
from arcflow import Agent, HitlConfig, Workflow

wf = (
    Workflow("support_escalation", runtime="http://localhost:8080")
    .enable_recovery()
    .step(Agent(name="triage", role="support", instructions="Summarize the ticket."))
    .step(
        Agent(name="lead", role="manager", instructions="Review before customer reply."),
        hitl=HitlConfig(approval_key="manager_signoff", timeout_seconds=86400),
    )
)
```

RCS JSON uses a `hitl` object on the step definition with `approval_key` and `timeout_seconds`. Optional `interrupt: true` is the default in the Python helper.

## Compliance notes

- Interrupt payloads on `GET /v1/runs/{id}` expose `approval_key`, `expires_at`, and optional `step_index`, not full conversation text.
- Trace exports remain metadata-only under SEC-1. See [SEC-1 rules](../observability/sec-1-rules.md).
- Approver interfaces should authenticate to the server (Bearer API key) or Relay-scoped keys when proxying approve calls.

## Related pages

- [Configuring interrupts](configuring-interrupts.md) for step-level fields and approval key design
- [Approve and reject](approve-and-reject.md) for the HTTP approval API
- [Recovery and resume](../reliability/recovery-and-resume.md) for Postgres persistence
- [Execution model](../../concepts/execution-model.md) for the full run state machine

## Source

Derived from [ARCFLOW-FULL-CAPABILITIES-REFERENCE.md](../../../docs/_draft/ARCFLOW-FULL-CAPABILITIES-REFERENCE.md) §8, Appendix E, Appendix F; `runtime/arcflow-core/src/human/`, `server/arcflow-server/src/handlers/approve.rs`, `examples/hitl/`.
