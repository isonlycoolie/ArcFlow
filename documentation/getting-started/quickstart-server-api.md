# Server API quickstart

**Audience:** `[platform]`

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
