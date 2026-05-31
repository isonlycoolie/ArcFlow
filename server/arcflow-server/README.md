# ArcFlow server deployment

## Quick start

```bash
export ARCFLOW_SERVER_API_KEY=dev-secret
export ARCFLOW_ADMIN_API_KEY=dev-admin
export ARCFLOW_POSTGRESQL_URL=postgres://arcflow:arcflow@localhost:5432/arcflow

# Apply migrations (once per schema version)
cargo run -p arcflow-cli -- migrate up

# Run server
cargo run -p arcflow-server
```

## Docker

From repo root:

```bash
docker compose -f docker/docker-compose.server.yml up --build
```

Migrations run via the `arcflow-migrate` init service before the server starts.

Set `ARCFLOW_SERVER_API_KEY` and `ARCFLOW_ADMIN_API_KEY` in the environment before production use.

## Readiness

- `GET /health` — liveness (process up)
- `GET /ready` — 200 when Postgres is reachable and migrations are at head; 503 when degraded

## SDK remote mode

```python
wf = Workflow("demo", runtime="http://localhost:8080")
wf.step(agent).run("hello")
```

Set `ARCFLOW_SERVER_API_KEY` in the client environment to match the server.

## Load test

```bash
bash scripts/load-test-runs.sh
```
