# External webhook example

**Audience:** `[developer]` `[compliance]`

This walkthrough posts an external binding outcome to the ArcFlow server using HMAC-authenticated callbacks. The stub script simulates a Playwright or government portal step reporting success, failure, or needs-input back into the run state machine.

Primary script: [`examples/external/playwright_stub_callback.py`](../../examples/external/playwright_stub_callback.py).

## What this example demonstrates

External steps delegate work outside the agent loop (browser automation, legacy APIs, manual portals). When the external system finishes, it calls the server callback endpoint through `report_outcome()`. The workflow advances based on structured outcome status, not raw page HTML.

Pair this script with published workflows that declare `ExternalBindingConfig` on the dashboard or server admin API. See [external callbacks](../guides/external-integrations/external-callbacks.md) and [webhook security](../guides/external-integrations/webhook-security.md).

## Prerequisites

| Item | Required |
|------|----------|
| Running `arcflow-server` | Docker compose or local binary on port 8080 |
| Active run with external binding | From published workflow or SDK with binding id |
| Env | `ARCFLOW_BASE_URL` (default `http://localhost:8080`) |
| Server auth | Callback signing secret configured per binding |
| Tutorial track | [Track E](../tutorials/track-e-hitl-and-external.md) |

Static product context: [`examples/static/online-application-chatbot/`](../../examples/static/online-application-chatbot/) references this callback pattern in production intake flows.

## Step 1: Obtain a run id with external binding

Start the server:

```bash
docker compose -f docker/docker-compose.server.yml up -d
```

Create or run a workflow whose step declares an external binding (default binding id in the stub: `gov_portal_submit`). Copy `run_id` from create response or SDK interrupt output when the external step activates.

For structure-only tests without live server:

```bash
pytest examples/static/online-application-chatbot/test_e2e.py -q
```

## Step 2: Report outcome with the stub script

Success path:

```bash
python examples/external/playwright_stub_callback.py \
  --run-id YOUR_RUN_ID \
  --binding-id gov_portal_submit \
  --status success
```

Needs-input path (validation error simulation):

```bash
python examples/external/playwright_stub_callback.py \
  --run-id YOUR_RUN_ID \
  --status needs_input \
  --error-code INVALID_NAME
```

Script core:

```python
from arcflow.external import report_outcome

resp = report_outcome(
    run_id,
    binding_id,
    {"status": status, "error_code": error_code},
    base_url=base_url,
)
```

## Step 3: Poll run status

```bash
curl -s "http://localhost:8080/v1/runs/YOUR_RUN_ID" \
  -H "X-ArcFlow-Api-Key: dev-secret"
```

After success outcome, expect progression toward `Completed` or next step state depending on workflow graph.

## Expected output

Script stdout prints JSON response from the server on success:

```
{"run_id": "...", "status": "Running", ...}
```

On signing or auth failure, stderr shows `[ArcFlow] callback failed: ...` and exit code 1.

## Trace events you should see
