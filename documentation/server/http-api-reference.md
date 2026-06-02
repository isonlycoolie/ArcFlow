
# HTTP API reference

Authoritative route list for `arcflow-server`. Matches [HTTP API reference](../server/http-api-reference.md).

This page reflects the implemented HTTP API.

**Server streaming (deferred):** `GET /v1/runs/{run_id}/events` (SSE) is **deferred**. Not listed below.

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
 "status": "Running"
}
```

Errors: **400** validation, **401** auth, **403** scoped key workflow denied, **503** Postgres unavailable.

### GET /v1/runs/{run_id}

Poll status, result, error, interrupt.

```bash
curl -s "http://localhost:8080/v1/runs/7c9e6679-7425-40de-944b-e07fc1f90ae7" \
 -H "Authorization: Bearer dev-secret"
```

**200** completed:

```json
{
 "run_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
 "status": "Completed",
 "result": { "output": "Final answer text", "step_outputs": {} },
 "error": null,
 "interrupt": null
}
```

**200** interrupted (HITL):

```json
{
 "run_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
 "status": "Interrupted",
 "result": null,
 "error": null,
 "interrupt": {
 "approval_key": "manager_signoff",
 "step_id": "00000000-0000-4000-8000-000000000030",
 "metadata": { "summary_bytes": 256 }
 }
}
```

### GET /v1/runs/{run_id}/trace

metadata-only trace `ExecutionTrace` JSON.

```bash
curl -s "http://localhost:8080/v1/runs/7c9e6679-7425-40de-944b-e07fc1f90ae7/trace" \
 -H "Authorization: Bearer dev-secret"
```

### POST /v1/runs/{run_id}/approve/{approval_key}

Resume or reject HITL interrupt.

```bash
curl -s -X POST "http://localhost:8080/v1/runs/RUN_ID/approve/manager_signoff" \
 -H "Authorization: Bearer dev-secret" \
 -H "Content-Type: application/json" \
 -d '{"approved": true, "data": {}}'
```

**200** on approve:

```json
{ "run_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7", "status": "Running" }
```

Reject with `"approved": false` yields terminal `Failed` (HumanRejected).

### POST /v1/runs/{run_id}/external/{binding_id}

External system callback. Body: `ExternalOutcomeReport` JSON. Verify `X-ArcFlow-Signature` when `ARCFLOW_WEBHOOK_SECRET` is set.

```bash
curl -s -X POST "http://localhost:8080/v1/runs/RUN_ID/external/binding-uuid" \
 -H "Authorization: Bearer dev-secret" \
 -H "Content-Type: application/json" \
 -H "X-ArcFlow-Signature: sha256=..." \
 -d '{"status":"completed","output":{"ticket_id":"T-99"}}'
```

## Workflow registry (runtime API key)

### PUT /v1/workflows/{name}/versions/{version}

Publish a versioned workflow definition.

```bash
curl -s -X PUT "http://localhost:8080/v1/workflows/chat/versions/1.0.0" \
 -H "Authorization: Bearer dev-secret" \
 -H "Content-Type: application/json" \
 -d '{"definition":{"id":"00000000-0000-4000-8000-000000000099","name":"chat","execution_mode":"linear","steps":[]},"agents":[]}'
```

Note: handler expects publish payload per `handlers/registry.rs` (definition + agents structure).

### GET /v1/workflows/{name}/versions/{version}

Fetch exact version.

```bash
curl -s "http://localhost:8080/v1/workflows/chat/versions/1.0.0" \
 -H "Authorization: Bearer dev-secret"
```

### GET /v1/workflows/{name}/resolve?range={semver}

Resolve semver range to highest matching version.

```bash
curl -s "http://localhost:8080/v1/workflows/chat/resolve?range=%5E1.0.0" \
 -H "Authorization: Bearer dev-secret"
```

Response:

```json
{ "name": "chat", "version": "1.0.2", "definition": { } }
```

### POST /v1/workflows/{name}/aliases/{alias}

Point alias (e.g. `latest`) at a version.

```bash
curl -s -X POST "http://localhost:8080/v1/workflows/chat/aliases/latest" \
 -H "Authorization: Bearer dev-secret" \
 -H "Content-Type: application/json" \
 -d '{"version": "1.0.2"}'
```

## Deprecated

### POST /v1/workflows/run

Legacy single-shot run. Prefer `POST /v1/runs`. Still registered for backward compatibility.

```bash
curl -s -X POST http://localhost:8080/v1/workflows/run \
 -H "Authorization: Bearer dev-secret" \
 -H "Content-Type: application/json" \
 -d @run-payload.json
```

## Admin (admin API key)

Auth: `Authorization: Bearer <ARCFLOW_ADMIN_API_KEY>`.

### POST /v1/admin/sites

Create site; `site_token` returned once.

```bash
curl -s -X POST http://localhost:8080/v1/admin/sites \
 -H "Authorization: Bearer dev-admin" \
 -H "Content-Type: application/json" \
 -d '{
 "display_name": "Acme Support",
 "allowed_origins": ["https://www.acme.com"],
 "rate_limit_rpm": 60,
 "allow_inline": false,
 "default_workflow_name": "chat",
 "chat_instructions": "Answer from knowledge only."
 }'
```

Response:

```json
{
 "site_id": "site_abc123",
 "relay_url": "http://localhost:8090/v1/sites/site_abc123",
 "site_token": "st_live_...",
 "kb_namespace": "site-site_abc123-kb"
}
```

### GET /v1/admin/sites/{site_id}

```bash
curl -s "http://localhost:8080/v1/admin/sites/site_abc123" \
 -H "Authorization: Bearer dev-admin"
```

### PATCH /v1/admin/sites/{site_id}

Update origins, rate limits, defaults.

```bash
curl -s -X PATCH "http://localhost:8080/v1/admin/sites/site_abc123" \
 -H "Authorization: Bearer dev-admin" \
 -H "Content-Type: application/json" \
 -d '{"allowed_origins":["https://app.acme.com"],"rate_limit_rpm":120}'
```

### POST /v1/admin/sites/{site_id}/tokens/rotate

Invalidate prior token; returns new token once.

```bash
curl -s -X POST "http://localhost:8080/v1/admin/sites/site_abc123/tokens/rotate" \
 -H "Authorization: Bearer dev-admin"
```

### POST /v1/admin/sites/{site_id}/knowledge/ingest

```bash
curl -s -X POST "http://localhost:8080/v1/admin/sites/site_abc123/knowledge/ingest" \
 -H "Authorization: Bearer dev-admin" \
 -H "Content-Type: application/json" \
 -d '{"text":"Our refund policy allows returns within 30 days.","key":"faq-refunds"}'
```

### POST /v1/admin/sites/{site_id}/workflows/chat/publish

Publish chat workflow to registry for the site.

```bash
curl -s -X POST "http://localhost:8080/v1/admin/sites/site_abc123/workflows/chat/publish" \
 -H "Authorization: Bearer dev-admin" \
 -H "Content-Type: application/json" \
 -d '{"instructions":"Answer only from ingested knowledge. Say when unsure.","version":"1.0.1"}'
```

Full admin contract: [Admin API reference](../operator/admin-api-reference.md).

## Debug (localhost, ARCFLOW_DEBUG)

Feature-gated. Not for production.

- `POST /v1/debug/runs/start`
- `GET /v1/debug/runs/{id}/state`
- `POST /v1/debug/runs/{id}/continue`

## Common HTTP codes

| Code | Typical cause |
|------|---------------|
| 401 | Missing or wrong API key |
| 403 | Scoped key, origin (Relay), or inline workflow denied |
| 429 | Relay rate limit |
| 503 | Postgres unavailable |

## Related pages

- [Authentication](authentication.md) — API keys and scoped access
- [Run state machine](run-state-machine.md) — polling and lifecycle states
- [Idempotency](idempotency.md) — safe create retries
- [Request path](../relay/request-path.md) — browser proxy routes
