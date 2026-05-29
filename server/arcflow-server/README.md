# ArcFlow server deployment

## Quick start

```bash
export ARCFLOW_SERVER_API_KEY=dev-secret
export ARCFLOW_POSTGRESQL_URL=postgres://arcflow:arcflow@localhost:5432/arcflow

# Apply migrations (once)
psql "$ARCFLOW_POSTGRESQL_URL" -f runtime/arcflow-core/migrations/002_recovery_v2.sql
psql "$ARCFLOW_POSTGRESQL_URL" -f runtime/arcflow-core/migrations/003_arcflow_runs.sql
psql "$ARCFLOW_POSTGRESQL_URL" -f runtime/arcflow-core/migrations/004_human_approvals.sql
psql "$ARCFLOW_POSTGRESQL_URL" -f runtime/arcflow-core/migrations/005_trace_events.sql

# Run server
cargo run -p arcflow-server
```

## Docker

From repo root:

```bash
docker compose -f docker/docker-compose.server.yml up --build
```

Set `ARCFLOW_SERVER_API_KEY` in the compose file or environment before production use.

## SDK remote mode

```python
wf = Workflow("demo", runtime="http://localhost:8080")
wf.step(agent).run("hello")
```

```typescript
const wf = new Workflow({ name: "demo", runtime: "http://localhost:8080" });
await wf.step(agent).run("hello");
```

Set `ARCFLOW_SERVER_API_KEY` in the client environment to match the server.

## Load test

With the server running:

```bash
bash scripts/load-test-runs.sh
```

Environment variables: `ARCFLOW_LOAD_CONCURRENCY` (default 100), `ARCFLOW_LOAD_MAX_P99_MS` (default 500).
