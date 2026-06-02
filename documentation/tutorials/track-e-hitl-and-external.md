# Track E: HITL and external integrations


Track E combines human approval interrupts with external webhook callbacks on the server. You drive a run from `Running` through `Interrupted` to `Completed`, and post structured external outcomes with verified auth.

## Goal

Run a workflow that interrupts for human approval, approve it via API, and receive an external webhook callback outcome. Verify the full state machine from `Interrupted` to `Completed`.

## Prerequisites

| Item | Required |
|------|----------|
| Server stack | Docker compose with Postgres |
| API keys | `ARCFLOW_SERVER_API_KEY=dev-secret` |
| Tracks | [Track B](track-b-server-api.md) for HTTP basics; [Track A](track-a-first-workflow.md) for SDK |
| HITL example | [HITL approval flow](../examples/hitl-approval-flow.md) |
| External example | [External webhook](../examples/external-webhook.md) |
| Guides | [HITL overview](../guides/human-in-the-loop/hitl-overview.md), [external callbacks](../guides/external-integrations/external-callbacks.md), [webhook security](../guides/external-integrations/webhook-security.md) |

## Part 1: Human approval

### Step 1: Start server

```bash
docker compose -f docker/docker-compose.server.yml up -d --build
curl -sf http://localhost:8080/ready
```

### Step 2: Run expense approval workflow

```bash
export ARCFLOW_RUNTIME=http://localhost:8080
export ARCFLOW_SERVER_API_KEY=dev-secret
python examples/hitl/expense_approval.py
```

Capture `run_id` from interrupt output.

### Step 3: Approve

```bash
bash examples/hitl/approve_cli.sh YOUR_RUN_ID
```

Poll until HTTP status is `Completed`:

```bash
curl -s "http://localhost:8080/v1/runs/YOUR_RUN_ID" \
 -H "X-ArcFlow-Api-Key: dev-secret"
```

Pass criteria: transition `Interrupted` then `Completed` with accounting step executed.

## Part 2: External callback

### Step 4: Run or create workflow with external binding

Use a published workflow with external step (see [`online-application-chatbot`](../examples/static-chat-widget.md) for shape) or SDK workflow declaring binding id `gov_portal_submit`.

Obtain `run_id` when external step activates.

### Step 5: Post outcome

```bash
python examples/external/playwright_stub_callback.py \
 --run-id YOUR_RUN_ID \
 --binding-id gov_portal_submit \
 --status success
```

For validation retry flows, use `--status needs_input --error-code INVALID_NAME`.

### Step 6: Verify run progression

Poll run status and trace. External success should advance workflow state without exposing secrets in trace export.

## Verification checklist

| Check | HITL | External |
|-------|------|----------|
| Interrupt state | `Interrupted` before approve | N/A |
| Approve API | 200 and resume | N/A |
| Terminal success | `Completed` | Run advances after valid outcome |
| Trace | HITL interrupt metadata | Step completion after callback |
| Security | Server key on approve only | HMAC on callback per binding |

## Expected output

HITL first phase prints approval instructions with run id. After approve, completed run with non-empty result. External stub prints server JSON response on success.

## Trace events you should see

| Phase | Events |
|-------|--------|
| HITL | `WorkflowStarted`, submit `StepCompleted`, interrupt metadata, post-approve steps, `WorkflowCompleted` |
| External | Step waiting state, `StepCompleted` after outcome, terminal workflow events |

All exports remain metadata-only trace. See [trace event reference](../guides/observability/trace-event-reference.md).

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| No interrupt | Recovery disabled | `.enable_recovery()` and Postgres |
| Approve 404 | Wrong approval key | Use `manager_approval` from expense script |
| Callback auth failure | Secret mismatch | Align binding HMAC secret |
| Callback from browser | Architecture violation | Call from backend only |

## What you learned

Track E covers operator-facing controls: pausing automation for human decisions and re-entering flows from external systems with verified callbacks. Compliance reviewers focus on auth boundaries and metadata-only traces here.

## Next tracks

| Track | Focus |
|-------|-------|
| F | Static product embedding |
| G | Operator migrations and CLI |
| Level 2 cert | Combined support workflow project |
