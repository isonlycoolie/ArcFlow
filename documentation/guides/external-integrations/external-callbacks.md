**Audience:** `[developer]` `[platform]`

# External callbacks

External bindings let a workflow wait for work outside the Rust engine: browser automation, payment webhooks, scheduled jobs, or custom integrators. When a bound step completes, the runtime emits `ExternalBindingStarted`, sets the run to `Interrupted`, and waits for a signed callback on the server.

This guide covers workflow configuration with `ExternalBindingConfig`, the callback lifecycle, and reporting outcomes with **ExternalOutcome.report** (Python SDK symbol: `report_outcome`).

## Workflow-level bindings

Declare bindings on the workflow definition, not on individual agents:

```json
{
  "external_bindings": [
    {
      "id": "payment_webhook",
      "kind": "http_callback",
      "attach_to_step_id": "00000000-0000-4000-8000-000000000050",
      "mode": "async_callback",
      "outcome_schema": {
        "type": "object",
        "properties": {
          "status": { "enum": ["success", "failed", "needs_input"] }
        },
        "required": ["status"]
      },
      "recovery": {
        "max_retries": 3,
        "on_needs_input": "agent_reask",
        "on_fatal": "fail_run"
      }
    }
  ]
}
```

| Field | Purpose |
|-------|---------|
| `id` | Binding identifier in callback URL and traces |
| `kind` | Integrator type (`browser_automation`, `schedule_trigger`, `custom`, or `http_callback` in JSON payloads) |
| `attach_to_step_id` | Step UUID that triggers the wait after step agent work |
| `mode` | `async_callback` (default) waits for HTTP callback; `sync_tool` completes in-process |
| `outcome_schema` | JSON Schema for callback body validation |
| `recovery` | Policy when outcome is `failed` or `needs_input` |

Runs with external bindings require `recovery_enabled: true` and Postgres, same as HITL.

## Python: ExternalBindingConfig

```python
from arcflow import Workflow, Agent
from arcflow.external import ExternalBindingConfig, report_outcome

STEP_ID = "550e8400-e29b-41d4-a716-446655440000"

binding = ExternalBindingConfig(
    "gov_portal_submit",
    attach_to_step_id=STEP_ID,
    kind="browser_automation",
    mode="async_callback",
    recovery={
        "max_retries": 2,
        "on_needs_input": "agent_reask",
        "on_fatal": "hitl_escalate",
    },
)

wf = Workflow("online_application", runtime="http://localhost:8080")
wf.enable_recovery()
wf.add_external_binding(binding)
wf.step(Agent(name="applicant", role="user", instructions="Complete the form."))
```

Publish the workflow to the server with `external_bindings` included in the workflow JSON sent to `POST /v1/runs`, or register via the workflow registry when using `workflow_ref`.

## Runtime lifecycle

1. Engine executes the attached step (agent invocation, tools, memory).
2. On step completion, engine emits `ExternalBindingStarted` and sets run status `Interrupted`.
3. Integrator performs external work (Playwright, payment API, human form).
4. Integrator calls `POST /v1/runs/{run_id}/external/{binding_id}` with signed `ExternalOutcomeReport` JSON.
5. Engine validates schema, applies recovery policy, emits `ExternalBindingCompleted` or `ExternalBindingFailed`, and resumes or fails the run.

Trace events are metadata-only (binding id, duration, error codes). See [trace event reference](../observability/trace-event-reference.md).

## ExternalOutcome.report

Post outcomes from integrator code with the SDK helper exported as `report_outcome` from `arcflow.external` (also re-exported from `arcflow`). Documentation refers to this operation as **ExternalOutcome.report**.

```python
from arcflow.external import ExternalBindingConfig, report_outcome

# ExternalOutcome.report
response = report_outcome(
