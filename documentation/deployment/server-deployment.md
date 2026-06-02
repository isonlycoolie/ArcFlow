
# Server deployment

`arcflow-server` is the HTTP runtime for workflow execution, registry, admin sites, traces, HITL approve, and external callbacks. This guide covers bare-metal and container deployment beyond Compose specifics.

## Prerequisites

| Requirement | Notes |
|-------------|-------|
| Postgres 16+ | Required for `POST /v1/runs`, admin, registry, sites |
| Qdrant | Required for vector memory and site knowledge ingest |
| LLM provider key | At least one of `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `GEMINI_API_KEY` |
| Embedding provider | Not `stub` in production when RAG is used |

## Minimum environment

| Variable | Required | Purpose |
|----------|----------|---------|
| `ARCFLOW_SERVER_API_KEY` | Yes | Protects `/v1/runs`, registry, trace routes |
| `ARCFLOW_ADMIN_API_KEY` | Yes for admin | Protects `/v1/admin/*` |
| `ARCFLOW_POSTGRESQL_URL` | Yes for server runs | Connection pool and persistence |
| `OPENAI_API_KEY` (or peer) | Yes for real LLM | Agent execution |

Generate keys:

```bash
export ARCFLOW_SERVER_API_KEY="$(openssl rand -hex 32)"
export ARCFLOW_ADMIN_API_KEY="$(openssl rand -hex 32)"
```

Use distinct values for server and admin keys. Minimum practical length is 32 hex characters (256 bits of entropy).

## Startup sequence

1. **Environment validation:** `main.rs` exits if `ARCFLOW_SERVER_API_KEY` is unset.
2. **Postgres pool:** `AppState::from_env` connects when `ARCFLOW_POSTGRESQL_URL` is set; pool size from `ARCFLOW_PG_MAX_CONNECTIONS` (default 10).
3. **Migration:** Run `arcflow migrate up` before first traffic, or rely on server auto-migrate and confirm via `/ready`.
4. **Listen:** Binds `0.0.0.0:ARCFLOW_PORT` (default 8080).
5. **Readiness:** Load balancers should gate on `GET /ready` returning 200.

```bash
arcflow migrate up
arcflow-server
```

Container entrypoint: `arcflow-server` as user `arcflow` (uid 1000).

## Authentication model

| Route group | Auth |
|-------------|------|
| `GET /health`, `GET /ready` | None |
| `/v1/runs`, registry, trace | `Authorization: Bearer <ARCFLOW_SERVER_API_KEY>` |
| `/v1/admin/*` | `Authorization: Bearer <ARCFLOW_ADMIN_API_KEY>` |
| `/v1/debug/*` | Localhost + `ARCFLOW_DEBUG=true` (debug feature build) |

Scoped runtime keys via `ARCFLOW_STATIC_RUNTIME_KEYS` limit which workflows a key may start. Relay upstream calls use scoped keys derived from site configuration.

## Postgres requirement behavior

`POST /v1/runs` returns **503** when Postgres is unset or the pool is unavailable. Embedded SDK runs on a developer laptop do not require Postgres unless recovery or registry features are enabled.

Size the pool: `(server_replicas × ARCFLOW_PG_MAX_CONNECTIONS) < postgres max_connections`.

## CORS

`ARCFLOW_CORS_ORIGINS` accepts a comma-separated allowlist for browser-direct server calls. Production static sites should use Relay instead of exposing the server API key in the browser.

## Request limits

Admin and runtime routes enforce a 1 MiB request body limit. Large knowledge ingest should chunk text in multiple admin calls if approaching the limit.

## Graceful shutdown

The server uses Axum/Tokio `serve` on the main listener. On **SIGTERM** (Docker stop, Kubernetes preStop), the process exits when in-flight requests complete per Tokio shutdown semantics. For zero-downtime deploys, drain the load balancer before sending SIGTERM.

Recommended Kubernetes pattern:

```yaml
lifecycle:
 preStop:
 exec:
 command: ["sleep", "5"]
terminationGracePeriodSeconds: 30
```

## Debug endpoints

When built with `debug-endpoints` feature and `ARCFLOW_DEBUG=true`, `/v1/debug/runs/*` routes are merged. Keep `ARCFLOW_DEBUG` unset or `false` in production. Debug routes are intended for localhost troubleshooting only.

## External callbacks

Set `ARCFLOW_WEBHOOK_SECRET` before enabling external binding workflows. Without it, external callback handlers reject requests. See [Webhook HMAC](../security/webhook-hmac.md).

## Verification after deploy

```bash
curl -sf http://localhost:8080/health
curl -sf http://localhost:8080/ready

curl -sf -X POST http://localhost:8080/v1/runs \
 -H "Authorization: Bearer $ARCFLOW_SERVER_API_KEY" \
 -H "Content-Type: application/json" \
 -d '{"workflow":{"id":"00000000-0000-4000-8000-000000000001","name":"smoke","execution_mode":"linear","steps":[]},"agents":[],"input":"hi","exec_config":{"recovery_enabled":false}}'
```

Adjust workflow JSON to match a valid minimal definition in your environment.

Load test reference: `bash scripts/load-test-runs.sh`.

## Backup

Back up Postgres (runs, recovery, registry, sites, trace events). Qdrant volumes hold vector data; back up `arcflow_qdrant_data` or use Qdrant snapshot APIs for RAG-heavy deployments.

## Related pages

- [Health and readiness](health-and-readiness.md)
- [Migrations runbook](migrations-runbook.md)
- [Environment variables reference](environment-variables-reference.md)
- [API key management](../security/api-key-management.md)
- [Production checklist](production-checklist.md)
