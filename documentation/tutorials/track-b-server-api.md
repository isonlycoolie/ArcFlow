# Track B: Server API

**Audience:** `[platform]`

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
