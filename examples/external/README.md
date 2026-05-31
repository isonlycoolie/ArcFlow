# External Portal Callback

## Problem

**CityPermits.gov** (stub) accepts online license applications only after a **legal name match** on their portal. Your ArcFlow workflow:

1. Collects applicant data in chat
2. Triggers an **external binding** to pre-fill the government form via automation (Playwright, RPA, etc.)
3. Waits for a signed webhook callback with `success`, `failed`, or `needs_input`

When the portal rejects a name format, the workflow must branch to ask the user for correction, not fail silently.

## Who this is for

| Role | Goal |
|------|------|
| **Integration engineer** | Wire RPA/Playwright worker to ArcFlow external callback |
| **Platform engineer** | HMAC-verified `POST /v1/runs/{id}/external/{binding_id}` |

## What ArcFlow demonstrates

- CLI stub posting `ExternalOutcomeReport` via `ExternalOutcome.report`
- Status values: `success`, `failed`, `needs_input`
- Binding id default: `gov_portal_submit` (matches static intake examples)

## Prerequisites

```bash
pip install -e sdk-python
docker compose -f docker/docker-compose.server.yml up -d
export ARCFLOW_WEBHOOK_SECRET=dev-webhook-secret
export ARCFLOW_BASE_URL=http://localhost:8080
```

Start a run from [static/online-application-chatbot/](../static/online-application-chatbot/) or server API with `external_bindings` defined.

## Run callback stub

```bash
python examples/external/portal_outcome_callback.py \
  --run-id <RUN_ID> \
  --status needs_input \
  --error-code INVALID_NAME
```

## Verify

- Server accepts callback when HMAC signature matches secret
- Trace includes `ExternalBindingCompleted` or `ExternalBindingFailed`
- Workflow resumes or prompts user based on `needs_input`

## Production notes

- Rotate `ARCFLOW_WEBHOOK_SECRET`; never log raw webhook bodies
- Run Playwright workers in isolated network segment
- See [capabilities reference](../../docs/_draft/ARCFLOW-FULL-CAPABILITIES-REFERENCE.md) §9

## Files

| File | Purpose |
|------|---------|
| [`portal_outcome_callback.py`](portal_outcome_callback.py) | Post outcome to server |
