# Online Application Chatbot

Static-site example for Phase 2-Pro v2 conversation intake + external callback.

## Files

- `arcflow.schedule.yaml` — schedule manifest (validate only)
- `sample_run.json` — POST body shape reference (inline RCS via `@arcflow/static`)
- `test_e2e.py` — pytest validation

## Static SDK

Use `@arcflow/static` to build the same payload:

```typescript
import { ArcFlowClient, StepForm, Workflow, resolveWorkflow } from "@arcflow/static";

const client = new ArcFlowClient({ baseUrl, apiKey, mode: "bff" });
const wf = resolveWorkflow("online_application", "1.0.0", baseUrl);
await client.runWorkflow(wf, "Apply for business license", {
  initialState: new StepForm().addTurn("user", "...").toInitialState(),
});
```

## Run tests

```bash
pytest examples/static/online-application-chatbot/test_e2e.py -q
```

## Live callback (optional)

```bash
export ARCFLOW_E2E=1 ARCFLOW_E2E_RUN_ID=<run_id>
pytest examples/static/online-application-chatbot/test_e2e.py -q -k live
```

## Playwright stub

```bash
python examples/external/playwright_stub_callback.py --run-id <run_id>
```
