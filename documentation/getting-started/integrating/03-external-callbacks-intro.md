# 03 External callbacks intro

**Audience:** `[developer]` `[platform]`

## Before you start

Complete [02 Server API first run](02-server-api-first-run.md) so the server stack, Bearer auth, and `Interrupted` run status are familiar. External callbacks require Postgres with `recovery_enabled: true`, same as HITL.

Read [External callbacks](../../guides/external-integrations/external-callbacks.md) when you need full JSON schema fields, recovery policies, and production HMAC setup.

## Concept

Some workflow steps delegate work outside the Rust engine: browser automation, payment processors, legacy APIs, or manual portals. ArcFlow models this with **external bindings** declared on the workflow definition. When the attached step finishes agent work, the engine emits `ExternalBindingStarted`, sets run status to `Interrupted`, and waits for a signed HTTP callback.

The integrator performs external work, then reports the outcome. Documentation calls this operation **ExternalOutcome.report**. The Python SDK exports the helper as `report_outcome` from `arcflow.external`. There is no Python type named `ExternalOutcome`; the server accepts `ExternalOutcomeReport` JSON.

Typical lifecycle:

1. Engine executes the bound step (agent, tools, memory).
2. Run moves to `Interrupted` with external binding metadata.
3. Integrator completes external work (Playwright, webhook handler, batch job).
4. Integrator calls `POST /v1/runs/{run_id}/external/{binding_id}` with signed outcome JSON.
5. Engine validates the outcome schema, applies recovery policy, and resumes or fails the run.

Runs with external bindings need `exec_config.recovery_enabled: true` (Python: `Workflow.enable_recovery()`).

## Example

Minimal Python workflow with one binding (server must be running):

```python
import os

os.environ.setdefault("ARCFLOW_SERVER_API_KEY", "dev-secret")
os.environ.setdefault("ARCFLOW_WEBHOOK_SECRET", "dev-webhook-secret")

from arcflow import Agent, Workflow
from arcflow.external import ExternalBindingConfig, report_outcome

STEP_ID = "550e8400-e29b-41d4-a716-446655440000"

binding = ExternalBindingConfig(
    "gov_portal_submit",
    attach_to_step_id=STEP_ID,
    kind="browser_automation",
    mode="async_callback",
)

wf = Workflow("online_application", runtime="http://localhost:8080")
wf.enable_recovery()
wf.add_external_binding(binding)
wf.step(Agent(name="applicant", role="user", instructions="Complete the form."))

result = wf.run("start application")
print(result.run_id, result.status)
```

When the run pauses, report success with **ExternalOutcome.report** (`report_outcome` in code):

```python
# ExternalOutcome.report
response = report_outcome(
    run_id="<run_id from interrupted run>",
    binding_id="gov_portal_submit",
    outcome={
        "status": "success",
        "fields": {"confirmation_number": "APP-2026-9912"},
    },
    base_url="http://localhost:8080",
)
print(response)
```

Outcome `status` values:

| status | Meaning |
|--------|---------|
| `success` | External work succeeded; engine resumes |
| `failed` | Fatal external error; recovery policy applies |
| `needs_input` | More input required; may trigger agent re-ask |

Required environment variables for the callback helper:

| Variable | Purpose |
|----------|---------|
| `ARCFLOW_SERVER_API_KEY` | Bearer auth on callback POST |
| `ARCFLOW_WEBHOOK_SECRET` | HMAC signing secret (must match server) |

If `ARCFLOW_WEBHOOK_SECRET` is unset on the server, external callback routes return **503**.

See `examples/external/playwright_stub_callback.py` for a CLI stub and `examples/static/online-application-chatbot/` for a static product pattern.

## Verify

| Check | Expected |
|-------|----------|
| Run with binding reaches `Interrupted` | Yes, after bound step completes |
| `report_outcome` with valid HMAC | Run resumes toward `Completed` |
| Trace after callback | `ExternalBindingCompleted` metadata event |
| Invalid signature | Rejected before outcome parsing |

Poll `GET /v1/runs/{run_id}` while waiting. When status is `Interrupted`, inspect the interrupt payload for binding context before posting the outcome.

## Next

[04 HITL approval intro](04-hitl-approval-intro.md) covers human approval gates, another `Interrupted` pattern that uses `POST /v1/runs/{run_id}/approve/{approval_key}` instead of external callbacks.

Deep dives: [Webhook security](../../guides/external-integrations/webhook-security.md), [Recovery and resume](../../guides/reliability/recovery-and-resume.md), [Track E: HITL and external](../../tutorials/track-e-hitl-and-external.md).

## Source

`sdk-python/arcflow/external.py`, `server/arcflow-server/src/handlers/external.rs`, `examples/external/`; [External callbacks](../../guides/external-integrations/external-callbacks.md); capabilities reference §9, §25.
