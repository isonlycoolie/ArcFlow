# 02 Server API first run


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
 "trace_id": "trace-7c9e6679",
 "status": "Running"
}
```

Poll until terminal status (replace `RUN_ID`):

```bash
curl -s "http://localhost:8080/v1/runs/RUN_ID" \
 -H "Authorization: Bearer dev-secret"
```

When complete, `status` is `Completed` and `result.output` holds the final text. Fetch the trace:

```bash
curl -s "http://localhost:8080/v1/runs/RUN_ID/trace" \
 -H "Authorization: Bearer dev-secret"
```

Expect lifecycle kinds such as `WorkflowStarted`, `StepCompleted`, and `WorkflowCompleted`. Trace exports are metadata-only under trace data policy: no prompts, tool payloads, or secrets.

Stop the stack when finished:

```bash
docker compose -f docker/docker-compose.server.yml down
```

Add `-v` only if you intend to wipe the Postgres volume.

## Verify

| Check | Expected |
|-------|----------|
| `/ready` HTTP code | `200` after migrations |
| Create run HTTP code | `201` with `run_id` |
| Poll until complete | `status` is `Completed` |
| Trace response | Non-empty `events` array with workflow lifecycle kinds |
| Stub path | No LLM API keys required |

Optional: send `Idempotency-Key: <uuid>` on create to deduplicate identical submissions within the server window.

## Next

[03 External callbacks intro](03-external-callbacks-intro.md) covers runs that pause at `Interrupted` until an external system posts an outcome.

For the full server quickstart with SDK client and bash poll loop, see [Server API quickstart](../quickstart-server-api.md).
