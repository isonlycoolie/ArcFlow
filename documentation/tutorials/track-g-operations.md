# Track G: Operations

**Audience:** `[operator]`

Track G focuses on running ArcFlow in production-like local conditions: schema migrations, readiness probes, trace export, and idempotent re-migration. No application workflow authoring required.

## Goal

Apply migrations, verify `/ready`, export trace via CLI, and confirm idempotent re-migration. Operator-focused tasks rather than SDK development.

## Prerequisites

| Item | Required |
|------|----------|
| Docker Compose | Server stack |
| Rust toolchain | For `arcflow-cli` from repo root |
| Prior reading | [Track B](track-b-server-api.md) for server basics |
| Guides | [execution traces](../guides/observability/execution-traces.md), [SEC-1 rules](../guides/observability/sec-1-rules.md) |

## Step 1: Start stack with migrate job

```bash
docker compose -f docker/docker-compose.server.yml up -d --build
```

Compose runs `arcflow-migrate` before `arcflow-server` starts. Confirm migrate container exited successfully:

```bash
docker compose -f docker/docker-compose.server.yml logs arcflow-migrate
```

## Step 2: Verify readiness

```bash
curl -sf http://localhost:8080/health
curl -s -o /dev/null -w "%{http_code}\n" http://localhost:8080/ready
```

Expect `/ready` **200** when Postgres is reachable and schema is at head. **503** indicates degraded state; do not route production traffic.

Document result:

```
ready_http_code=200
```

## Step 3: Create a run for trace export

Use [Track B](track-b-server-api.md) payload or quick curl create. Copy `run_id` after `Completed`.

```bash
curl -s -X POST http://localhost:8080/v1/runs \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer dev-secret" \
  -d @run-payload.json
```

## Step 4: Export trace via CLI

```bash
cargo run -p arcflow-cli -- trace YOUR_RUN_ID --format json --verbose
```

Compare CLI output to HTTP trace:

```bash
curl -s "http://localhost:8080/v1/runs/YOUR_RUN_ID/trace" \
  -H "Authorization: Bearer dev-secret"
```

Pass criteria: both exports list lifecycle events; no prompt text or secrets in JSON (SEC-1).

## Step 5: Idempotent re-migration

Run migrate again explicitly:

```bash
cargo run -p arcflow-cli -- migrate up
```

Or restart migrate service:

```bash
docker compose -f docker/docker-compose.server.yml run --rm arcflow-migrate
```

Second run should exit zero with no pending migrations. Schema version remains at head; no duplicate object errors.

Verify `/ready` still **200** after re-migration.

## Verification checklist

| Check | Expected |
|-------|----------|
| First migrate | Success, server starts |
