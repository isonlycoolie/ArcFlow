# Track B: Server API


Track B introduces the HTTP execution surface: Docker Compose, migrations, `POST /v1/runs`, status polling, and trace export. You complete the same linear stub pipeline as Track A, but through `arcflow-server` and Postgres persistence.

## Goal

Start the server via Docker Compose, confirm readiness, create a run with `POST /v1/runs`, poll until terminal status, and retrieve the execution trace with `GET /v1/runs/{run_id}/trace`.

## Prerequisites

| Item | Required |
|------|----------|
| Docker Compose v2 | Yes |
| `curl` | Yes (use `curl.exe` on Windows PowerShell if aliased) |
| API keys | Compose dev keys only (`dev-secret`, `dev-admin`) |
| Prior track | [Track A](track-a-first-workflow.md) recommended but not mandatory |
| Primary guide | [Server API quickstart](../getting-started/quickstart-server-api.md) |

Embedded SDK runs do not require Postgres. The server returns **503** on run create when the database pool is unavailable.

## Primary example

Two-step research pipeline payload (stub agents, no LLM keys). Same logical workflow as Track A with HTTP status strings in PascalCase.

## Step 1: Start the stack

```bash
docker compose -f docker/docker-compose.server.yml up -d --build
```

Services: Postgres (5432), `arcflow-migrate` (init), `arcflow-server` (8080), `arcflow-relay` (8090, unused in this track).

## Step 2: Confirm health and readiness

```bash
curl -sf http://localhost:8080/health
curl -s -o /dev/null -w "%{http_code}\n" http://localhost:8080/ready
```

Expect `/health` body OK and `/ready` HTTP **200**. **503** on `/ready` means migrations or Postgres are not ready yet; wait and retry.

## Step 3: Create a run

Save payload as `run-payload.json`:

```json
{
  "workflow": {
    "id": "00000000-0000-4000-8000-000000000099",
    "name": "research_pipeline",
    "execution_mode": "linear",
    "steps": [
      {
        "id": "00000000-0000-4000-8000-000000000001",
        "agent_id": "00000000-0000-4000-8000-000000000010",
        "order": 1
      },
      {
        "id": "00000000-0000-4000-8000-000000000002",
        "agent_id": "00000000-0000-4000-8000-000000000011",
        "order": 2
      }
    ]
  },
  "agents": [
    {
      "id": "00000000-0000-4000-8000-000000000010",
      "name": "researcher",
      "role": "research",
      "instructions": "Research the given topic thoroughly."
    },
    {
      "id": "00000000-0000-4000-8000-000000000011",
      "name": "writer",
      "role": "write",
      "instructions": "Write a clear summary of the research."
    }
  ],
  "input": "Analyze renewable energy trends",
  "exec_config": {
    "recovery_enabled": true,
    "workflow_timeout_secs": 300
  }
}
```

Submit:

```bash
curl -s -X POST http://localhost:8080/v1/runs \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer dev-secret" \
  -d @run-payload.json
```

Copy `run_id` from **201** response. Optional: add `Idempotency-Key: <uuid>` header for deduplication.

## Step 4: Poll until terminal status

```bash
RUN_ID="paste-uuid-here"
curl -s "http://localhost:8080/v1/runs/${RUN_ID}" \
  -H "Authorization: Bearer dev-secret"
```

Poll until `status` is `Completed`, `Failed`, `Cancelled`, or `Interrupted`. Stub runs typically finish in seconds.

Bash loop:

```bash
until STATUS=$(curl -s "http://localhost:8080/v1/runs/${RUN_ID}" \
  -H "Authorization: Bearer dev-secret" | python3 -c "import sys,json; print(json.load(sys.stdin)['status'])") \
  && [[ "$STATUS" == "Completed" || "$STATUS" == "Failed" ]]; do
  sleep 1
done
echo "terminal status: $STATUS"
```

Track B pass criteria: `Completed` with non-empty `result.output`.

## Step 5: Retrieve trace

```bash
curl -s "http://localhost:8080/v1/runs/${RUN_ID}/trace" \
  -H "Authorization: Bearer dev-secret"
```

Verify event kinds include `WorkflowStarted`, `StepCompleted`, and `WorkflowCompleted`. Payloads are SEC-1 metadata only.

Optional CLI:

```bash
cargo run -p arcflow-cli -- trace ${RUN_ID} --format json --verbose
```

## Step 6: Verification checklist

| Check | Expected |
|-------|----------|
| `/ready` | HTTP 200 |
| Create run | HTTP 201 with `run_id` |
| Final status | `Completed` |
| Trace export | Lifecycle kinds present |
| Auth without key | HTTP 401 on `/v1/runs` |

## Expected output

Create response shape:

```json
{
  "run_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
  "trace_id": "trace-7c9e6679",
  "status": "Running"
}
```

Completed run includes `result.output` string. HTTP uses PascalCase; Python SDK embedded runs use lowercase `completed` for the same state.

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| 503 on POST `/v1/runs` | Postgres down or migrations pending | Check `docker compose logs arcflow-migrate` and `/ready` |
| 401 Unauthorized | Missing or wrong API key | Use `dev-secret` from compose file |
| Stuck in `Running` | Server error or hung provider | Inspect server logs; stub should finish quickly |
| Empty trace | Wrong run id | Copy id from create response exactly |

## What you learned

Track B maps the embedded SDK loop to durable server execution: auth tiers, run persistence, polling contract, and trace HTTP export. Platform engineers use this surface for backends, cron, and services that should not embed the SDK.

## Next tracks

| Track | Focus |
|-------|-------|
| C | RAG and Qdrant |
| G | Migrations, `/ready`, operator CLI |
| E | HITL and external callbacks on server |

Stop the stack when finished:

```bash
docker compose -f docker/docker-compose.server.yml down
```
