# Online Application Chatbot (Production)

Multi-turn application intake on a static site, e.g. business license, permit, or onboarding forms. Workflow shape, steps, and external callbacks are **published from the dashboard**; the static frontend only drives conversation turns via Relay.

## Who does what

| Role | Work |
|------|------|
| **Dashboard user** | Publish `online_application` workflow, configure HITL/external bindings |
| **Frontend developer** | Chat UI, `runPublished()`, optional external callback handler on your backend |

## Production frontend

```typescript
import { ArcFlowClient, StepForm } from "@arcflow/static";

const client = new ArcFlowClient({
  baseUrl: import.meta.env.VITE_ARCFLOW_RELAY_URL,
  apiKey: import.meta.env.VITE_ARCFLOW_SITE_TOKEN,
  mode: "relay",
});

const form = new StepForm()
  .addTurn("user", "I need to apply for a business license")
  .addTurn("assistant", "What is the applicant legal name?");

await client.runPublished("online_application", "^1.0.0", "Apply for business license", {
  initialState: form.toInitialState(),
});
```

No inline `Workflow` or `Agent` definitions in the browser bundle.

## Env vars

```bash
VITE_ARCFLOW_RELAY_URL=https://relay.arcflow.app/v1/sites/s_abc123
VITE_ARCFLOW_SITE_TOKEN=st_live_xxxxxxxx
```

## Files in this example

| File | Purpose |
|------|---------|
| `arcflow.schedule.yaml` | Schedule manifest (validate only) |
| `sample_run.json` | Legacy payload shape reference, dashboard owns workflow definition in production |
| `test_e2e.py` | Structure and optional live callback tests |

## External callback (production)

When a step triggers an external binding (e.g. Playwright form fill), your **backend** receives the webhook and calls ArcFlow server with the server key, never from the static bundle:

```bash
python examples/external/portal_outcome_callback.py --run-id <run_id>
```

## Tests

```bash
pytest examples/static/online-application-chatbot/test_e2e.py -q
```

Live callback (requires running server):

```bash
export ARCFLOW_E2E=1 ARCFLOW_E2E_RUN_ID=<run_id>
pytest examples/static/online-application-chatbot/test_e2e.py -q -k live
```

## Related

- [Static examples index](../README.md)
- [RAG upload guide](../../ArcFlow_Improvement_Plans/arcflow-static-product-vision/10-rag-document-upload-guide.md), if this bot also uses knowledge base docs
