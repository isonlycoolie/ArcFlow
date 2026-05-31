# Server API quickstart


## Before you start

You should understand why the server exists. Read [01 Embedded SDK vs server](integrating/01-embedded-sdk-vs-server.md) if you have not already.

Embedded SDK runs do not require PostgreSQL. The server requires Postgres for `POST /v1/runs`; without a reachable pool the handler returns **503**.

## Concept

The HTTP server (`arcflow-server`) exposes workflow execution to backends, cron jobs, and services that should not embed the SDK. This guide starts the Docker stack, confirms readiness, creates a run with `POST /v1/runs`, polls until completion, and fetches the execution trace. The sample payload runs without LLM API keys.

The compose file also starts `arcflow-relay` on port **8090** for browser clients. This quickstart uses direct server curl; Relay is covered in [Static site chatbot](paths/static-site-chatbot.md).

## Prerequisites

| Item | Required |
|------|----------|
| Docker with Compose v2 | Yes |
| `curl` | Yes (on Windows use `curl.exe` in PowerShell) |
| Repository clone | Any path; commands assume repo root |

## Start the stack

From the repository root:

```bash
docker compose -f docker/docker-compose.server.yml up -d --build
```

Services brought up:

| Service | Port | Role |
|---------|------|------|
| `postgres` | 5432 | Run persistence and registry |
| `arcflow-migrate` | (init) | Applies schema once Postgres is healthy |
| `arcflow-server` | 8080 | HTTP API |
| `arcflow-relay` | 8090 | Browser proxy (not needed for this guide) |

Development keys: `ARCFLOW_SERVER_API_KEY=dev-secret`, `ARCFLOW_ADMIN_API_KEY=dev-admin`. Change these before any non-local deployment.

Migrations run automatically via `arcflow-migrate` before the server starts. For bare-metal deployment without Docker, run `cargo run -p arcflow-cli -- migrate up` once per schema version (see `server/arcflow-server/README.md`).

## Confirm health and readiness

```bash
curl -sf http://localhost:8080/health
curl -s -o /dev/null -w "%{http_code}\n" http://localhost:8080/ready
```

`/health` is liveness (process up). `/ready` returns **200** when Postgres is reachable and migrations are at head; **503** when degraded.

## Example: create a run

Save the payload as `run-payload.json` in your working directory:

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

Submit the run:

```bash
curl -s -X POST http://localhost:8080/v1/runs \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer dev-secret" \
  -d @run-payload.json
```

On Windows PowerShell:

```powershell
curl.exe -s -X POST http://localhost:8080/v1/runs `
  -H "Content-Type: application/json" `
  -H "Authorization: Bearer dev-secret" `
  -d "@run-payload.json"
```

Example **201** response:

```json
{
  "run_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
  "trace_id": "trace-7c9e6679",
  "status": "Running"
}
```

Copy `run_id` for polling. Optional: send `Idempotency-Key: <uuid>` on create to deduplicate identical submissions.

## Poll run status

Replace `RUN_ID` with the value from the create response:

```bash
curl -s http://localhost:8080/v1/runs/RUN_ID \
  -H "Authorization: Bearer dev-secret"
```

Repeat until `status` is terminal (`Completed`, `Failed`, `Cancelled`, or `Interrupted`). Sample workflows typically finish in seconds.

Example **200** when complete:

```json
{
  "run_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
  "status": "Completed",
  "result": {
    "output": "Final answer text",
    "step_outputs": {}
  },
  "error": null,
  "interrupt": null
}
```

Poll loop (bash):

```bash
RUN_ID="paste-uuid-here"
until STATUS=$(curl -s "http://localhost:8080/v1/runs/${RUN_ID}" \
  -H "Authorization: Bearer dev-secret" | python3 -c "import sys,json; print(json.load(sys.stdin)['status'])") \
  && [[ "$STATUS" == "Completed" || "$STATUS" == "Failed" ]]; do
  sleep 1
done
echo "terminal status: $STATUS"
```

Poll loop (PowerShell):

```powershell
$runId = "paste-uuid-here"
do {
  Start-Sleep -Seconds 1
  $body = curl.exe -s "http://localhost:8080/v1/runs/$runId" -H "Authorization: Bearer dev-secret"
  $status = ($body | ConvertFrom-Json).status
} until ($status -in @("Completed", "Failed"))
Write-Host "terminal status: $status"
```

HTTP API status strings use PascalCase. The embedded Python SDK exposes lowercase `completed` on `WorkflowResult`; both refer to the same terminal state.

## Retrieve the trace

```bash
curl -s "http://localhost:8080/v1/runs/RUN_ID/trace" \
  -H "Authorization: Bearer dev-secret"
```

The response is an `ExecutionTrace` JSON document with metadata-only events (SEC-1). Expect lifecycle kinds such as `WorkflowStarted`, `StepCompleted`, and `WorkflowCompleted`.

From the repo root you can also inspect a run with the CLI:

```bash
cargo run -p arcflow-cli -- trace RUN_ID --format json --verbose
```

## Auth tiers (summary)

| Routes | Auth |
|--------|------|
| `/health`, `/ready` | None |
| `/v1/runs`, registry, trace | `Authorization: Bearer <ARCFLOW_SERVER_API_KEY>` or `X-ArcFlow-Api-Key` |
| `/v1/admin/*` | Bearer `ARCFLOW_ADMIN_API_KEY` |
| `/v1/debug/*` | Localhost and `ARCFLOW_DEBUG=true` |

Full route index is in [HTTP API reference](../server/http-api-reference.md) and [HTTP API reference](../server/http-api-reference.md).

## SDK client pointing at the server

Save as `server_client.py`:

```python
import os

os.environ["ARCFLOW_SERVER_API_KEY"] = "dev-secret"

from arcflow import Agent, Workflow

wf = Workflow("demo", runtime="http://localhost:8080")
wf.step(Agent(name="writer", role="author", instructions="Summarize."))
result = wf.run("hello")
print(result.output)
```

Run after the Docker stack is up:

```bash
python server_client.py
```

## Stop the stack

```bash
docker compose -f docker/docker-compose.server.yml down
```

Add `-v` only if you intend to wipe the Postgres volume.

## Verify

| Check | Expected |
|-------|----------|
| `/ready` | HTTP 200 |
| Create run | HTTP 201 with `run_id` |
| Poll | Terminal `Completed` for sample payload |
| Trace | Lifecycle events present, no prompt text |

## Next

| Topic | Link |
|-------|------|
| Tutorial with verification checklist | [Track B: Server API](../tutorials/track-b-server-api.md) |
| HITL and external on server | [Integrating track](integrating/README.md) |
| Workflow registry and semver | [Workflow registry](../guides/workflows/workflow-registry.md) |
| Production compose | `docker/docker-compose.prod.yml`, `contracts/guides/deployment/self-hosted.md` |
| Full HTTP reference | [Server overview](../server/overview.md) |
| Static browser product | [Static site chatbot](paths/static-site-chatbot.md) |
