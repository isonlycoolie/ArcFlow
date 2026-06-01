# Self-hosted ArcFlow runtime

Run the HTTP server with secrets in the environment, not in the image. HTTP contract: [HTTP API reference](../../../server/http-api-reference.md).

## Deploy flow

1. Start Postgres (and Qdrant if you use vector memory).
2. Run migrations once per schema change: `arcflow migrate up` with `ARCFLOW_POSTGRESQL_URL` set.
3. Start `arcflow-server` only after migrations succeed.
4. Confirm `GET /ready` returns 200 before sending traffic.

With Docker Compose:

```bash
docker compose -f docker/docker-compose.server.yml up --build
```

The `arcflow-migrate` service runs `arcflow migrate up` before the server starts.

Production-like stack (Postgres + Qdrant + server + relay):

```bash
docker compose -f docker/docker-compose.prod.yml up --build
```

## Required environment

| Variable | Purpose |
|----------|---------|
| `ARCFLOW_SERVER_API_KEY` | Protects `/v1/*` runtime routes |
| `ARCFLOW_ADMIN_API_KEY` | Protects `/v1/admin/*` |
| `ARCFLOW_POSTGRESQL_URL` | Runs, recovery, registry, sites, traces |
| `ARCFLOW_EMBEDDING_PROVIDER` | Use a real provider in production (not `stub`) |

Keep `ARCFLOW_DEBUG` unset or `false` in production. Debug routes are localhost-only when enabled.

## Connection pool

`ARCFLOW_PG_MAX_CONNECTIONS` defaults to `10` on the server. Size the limit for `(replicas × limit) < Postgres max_connections`.

## Observability

Optional OTLP: see [observability-otel.md](../../../docker/observability-otel.md).

## Load test

```bash
bash scripts/load-test-runs.sh
```

## Backup

Back up the Postgres volume (or your managed instance). Recovery and run history live there.

## Meta-repo (optional private layout)

Submodule the OSS tree inside your platform repo: [meta-repo.md](meta-repo.md).
