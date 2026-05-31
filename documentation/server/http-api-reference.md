**Audience:** `[platform]` `[developer]`

# HTTP API reference

Authoritative route list for `arcflow-server` as registered in `server/arcflow-server/src/lib.rs` (2026-05-31). Matches capabilities reference **Appendix B**.

**K-10 drift:** [contracts/normative/runtime/server-api-v1.md](../../contracts/normative/runtime/server-api-v1.md) still documents only legacy routes. Do not use it as the integration source until promoted from draft. This page and Appendix B reflect implemented behavior.

**FP-2:** `GET /v1/runs/{run_id}/events` (SSE) is **deferred**. Not listed below.

Base URL examples use `http://localhost:8080`. Replace with your deployment host.

## Public

### GET /health

Liveness probe. No auth.

```bash
curl -sf http://localhost:8080/health
```

### GET /ready

Readiness: Postgres reachable and migrations at head. **200** when ready, **503** when degraded.

```bash
curl -s -o /dev/null -w "%{http_code}\n" http://localhost:8080/ready
```

## Runs (runtime API key)

Auth: `Authorization: Bearer <ARCFLOW_SERVER_API_KEY>` or `X-ArcFlow-Api-Key`.

### POST /v1/runs

Create and execute a workflow run. Requires Postgres.

Body (inline workflow):

```json
{
  "workflow": {
    "id": "00000000-0000-4000-8000-000000000099",
    "name": "demo",
    "execution_mode": "linear",
    "steps": [
      {
        "id": "00000000-0000-4000-8000-000000000001",
        "agent_id": "00000000-0000-4000-8000-000000000010",
        "order": 1
      }
    ]
  },
  "agents": [
    {
      "id": "00000000-0000-4000-8000-000000000010",
      "name": "writer",
      "role": "author",
      "instructions": "Summarize the input."
    }
  ],
  "input": "Analyze renewable energy trends",
  "exec_config": {
    "recovery_enabled": true,
    "workflow_timeout_secs": 300
  }
}
```

Or registry reference (do **not** send both inline workflow and `workflow_ref`):

```json
{
  "workflow_ref": { "name": "chat", "version": "^1.0.0" },
  "input": "Hello"
}
```

Optional header: `Idempotency-Key: <uuid>`.

```bash
curl -s -X POST http://localhost:8080/v1/runs \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer dev-secret" \
  -H "Idempotency-Key: 550e8400-e29b-41d4-a716-446655440000" \
  -d @run-payload.json
```

**201** response:

```json
{
  "run_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
  "trace_id": "trace-7c9e6679",
