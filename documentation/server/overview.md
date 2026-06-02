
# arcflow-server overview

`arcflow-server` is the HTTP front door to the ArcFlow engine for backends, cron jobs, and services that should not embed the Python or TypeScript SDK. One Rust binary wraps `arcflow-core` with Postgres-backed run persistence, a semver workflow registry, Human-in-the-Loop (HITL) endpoints, external callback ingress, and the admin API that powers the static product.

Choose the server when you need durable run records, registry-based workflow resolution, operator provisioning of sites, or centralized API keys. Choose an embedded SDK when a single process owns the workflow and Postgres is optional.

## What the server adds beyond the SDK

| Capability | Embedded SDK | arcflow-server |
|------------|:------------:|:--------------:|
| `POST /v1/runs` with inline workflow | Via SDK only | Yes |
| Run status polling over HTTP | No | Yes |
| Workflow registry (semver publish/resolve) | No | Yes |
| HITL approve endpoint | In-process only | Yes |
| External callback ingress with HMAC | In-process | Yes |
| Admin sites, knowledge ingest, chat publish | No | Yes |
| Trace export `GET /v1/runs/{id}/trace` | In-process store | Postgres-backed |

Postgres is **required** for `POST /v1/runs`. If `ARCFLOW_POSTGRESQL_URL` is unset or the pool is unavailable, create-run returns **503**. Embedded SDK runs do not need Postgres unless you enable recovery or registry features locally.

## Architecture

```text
Client (curl, backend, Relay)
 → arcflow-server :8080
 → middleware (API key / admin key)
 → handlers (runs, registry, admin)
 → arcflow-core WorkflowEngine
 → Postgres (runs, traces, registry, sites, recovery)
```

Relay is a separate binary (`arcflow-relay`) that proxies browser traffic to the server with site tokens and origin checks. See [relay/overview.md](../relay/overview.md).

## Authentication tiers

Two primary keys protect different route groups:

| Tier | Env var | Routes |
|------|---------|--------|
| Runtime | `ARCFLOW_SERVER_API_KEY` | `/v1/runs`, registry, trace, approve, external |
| Admin | `ARCFLOW_ADMIN_API_KEY` | `/v1/admin/*` |

Public probes: `/health` (liveness), `/ready` (Postgres + migrations at head). Debug routes require localhost and `ARCFLOW_DEBUG=true` when compiled with the debug feature.

Scoped runtime keys (`ARCFLOW_STATIC_RUNTIME_KEYS`) limit which workflow names a key may start. Relay upstream calls use these keys so browsers never hold the master server key.

Full auth detail: [authentication.md](authentication.md).

## CORS

`ARCFLOW_CORS_ORIGINS` accepts a comma-separated list of allowed browser origins for direct server calls. Production static sites should use Relay instead of exposing the server API key in the browser. CORS on the server is mainly for local development with `mode: "direct"` in the static SDK.

## Deferred: server SSE (streaming deferred)

`GET /v1/runs/{run_id}/events` (Server-Sent Events) is **not implemented**. Poll `GET /v1/runs/{run_id}` or fetch the trace. SDK `run_stream()` works in-process only..

## Contract drift (K-10)

When integrating, prefer [HTTP API reference](http-api-reference.md) or [Admin API reference](../operator/admin-api-reference.md). Implemented routes match `server/arcflow-server/src/lib.rs`.

## Quick verification

Start the local stack from the repo root:

```bash
docker compose -f docker/docker-compose.server.yml up -d --build
curl -sf http://localhost:8080/health
curl -s -o /dev/null -w "%{http_code}\n" http://localhost:8080/ready
```

Create a stub run (see [Server API quickstart](../getting-started/quickstart-server-api.md) for the full payload):

```bash
curl -s -X POST http://localhost:8080/v1/runs \
 -H "Content-Type: application/json" \
 -H "Authorization: Bearer dev-secret" \
 -d '{"workflow":{"id":"00000000-0000-4000-8000-000000000099","name":"demo","execution_mode":"linear","steps":[{"id":"00000000-0000-4000-8000-000000000001","agent_id":"00000000-0000-4000-8000-000000000010","order":1}]},"agents":[{"id":"00000000-0000-4000-8000-000000000010","name":"writer","role":"author","instructions":"Summarize."}],"input":"hello"}'
```

## Related pages

| Topic | Page |
|-------|------|
| Auth tiers and rotation | [authentication.md](authentication.md) |
| Full route reference | [http-api-reference.md](http-api-reference.md) |
| Run lifecycle | [run-state-machine.md](run-state-machine.md) |
| Idempotent create | [idempotency.md](idempotency.md) |
| Schema | [postgres-schema.md](postgres-schema.md) |
| Known gaps | [maturity-and-known-gaps.md](../concepts/maturity-and-known-gaps.md) |
