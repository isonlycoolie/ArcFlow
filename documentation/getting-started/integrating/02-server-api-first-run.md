# 02 Server API first run

**Audience:** `[platform]`

## Before you start

Read [01 Embedded SDK vs server](01-embedded-sdk-vs-server.md) so the Postgres requirement and auth headers make sense. You need Docker with Compose v2, `curl`, and a repository clone. On Windows PowerShell, use `curl.exe` if the `curl` alias maps to `Invoke-WebRequest`.

## Concept

`arcflow-server` wraps `arcflow-core` with HTTP routes for run creation, status polling, traces, registry, admin sites, HITL approve, and external callbacks. The development compose file starts Postgres, runs migrations once, and exposes the API on port **8080**.

Development keys from `docker/docker-compose.server.yml`:

| Variable | Default | Used for |
|----------|---------|----------|
| `ARCFLOW_SERVER_API_KEY` | `dev-secret` | `POST /v1/runs`, run poll, trace |
| `ARCFLOW_ADMIN_API_KEY` | `dev-admin` | `/v1/admin/*` routes |

Change these before any non-local deployment.

Auth on run routes accepts either `Authorization: Bearer <key>` or `X-ArcFlow-Api-Key: <key>`. `/health` and `/ready` require no auth. `/ready` returns **200** when Postgres is reachable and migrations are at head; **503** when degraded.

## Example

Start the stack from the repository root:

```bash
docker compose -f docker/docker-compose.server.yml up -d --build
```

Confirm readiness:

```bash
curl -sf http://localhost:8080/health
curl -s -o /dev/null -w "%{http_code}\n" http://localhost:8080/ready
```

Save this payload as `run-payload.json`:

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

Create the run:

```bash
curl -s -X POST http://localhost:8080/v1/runs \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer dev-secret" \
  -d @run-payload.json
```

Example **201** response:

```json
{
  "run_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
