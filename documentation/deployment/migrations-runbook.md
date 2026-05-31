**Audience:** `[platform]` `[operator]`

# Migrations runbook

Postgres schema for ArcFlow is versioned under `server/arcflow-server/migrations/`. Apply migrations before routing production traffic to a new server version. This runbook lists commands and recovery steps; schema details are in Appendix G.

Migrations path: `server/arcflow-server/migrations/`.

Normative recovery DDL (partial): [contracts/normative/runtime/recovery-schema-v1.sql](../../contracts/normative/runtime/recovery-schema-v1.sql).

## Migration inventory

| Migration | Tables / changes |
|-----------|------------------|
| `20240531000001` | `arcflow_recovery_state` (linear recovery) |
| `20240531000002` | Graph columns on recovery (`current_node_id`, `graph_iteration_count`, `pending_join`) |
| `20240531000003` | `arcflow_runs` (run status, result snapshot) |
| `20240531000004` | `arcflow_human_approvals`, HITL snapshot columns |
| `20240531000005` | `arcflow_trace_events` (SEC-1 persisted events) |
| `20240531000006` | `arcflow_workflows`, `arcflow_workflow_aliases` (semver registry) |
| `20240531000007` | `arcflow_sites`, `arcflow_site_tokens`, `arcflow_site_usage_daily` |

## Pre-deploy procedure

1. Snapshot Postgres (managed backup or `pg_dump`).
2. Set `ARCFLOW_POSTGRESQL_URL` to the target database.
3. Run migrate against staging first; verify `/ready` returns 200.
4. Run migrate against production during the maintenance window.
5. Deploy new server binary or container.
6. Confirm `/ready` before re-enabling traffic.

```bash
export ARCFLOW_POSTGRESQL_URL=postgres://user:pass@host:5432/arcflow
arcflow migrate up
curl -sf http://localhost:8080/ready
```

## CLI commands

| Command | Purpose |
|---------|---------|
| `arcflow migrate up` | Apply pending migrations |
| `arcflow migrate validate` | Verify schema matches expected version (CI) |

Docker one-shot (production compose):

```yaml
arcflow-migrate:
  build:
    dockerfile: server/arcflow-server/Dockerfile.migrate
  environment:
    ARCFLOW_POSTGRESQL_URL: ...
  restart: "no"
```

Server depends on `service_completed_successfully` for migrate.

## Auto-migrate vs explicit migrate

The server checks pending migrations on `/ready` via `arcflow_core::migrate::pending`. If migrations are pending, `/ready` returns **503** with `"reason": "migrations_pending"`.

You may rely on server startup to apply migrations in small deployments, but explicit `arcflow migrate up` before deploy is preferred for controlled rollouts and CI gates.

## Idempotency

Re-running `arcflow migrate up` on an up-to-date database is a no-op. Safe to run in CI and init containers.

Verification in tests:

```bash
cargo test -p arcflow-server migrations
```

## Fresh install

```bash
createdb arcflow   # or use managed Postgres
export ARCFLOW_POSTGRESQL_URL=postgres://arcflow:arcflow@localhost:5432/arcflow
arcflow migrate up
arcflow migrate validate
```

Start server only after validate succeeds.

## Upgrade (version bump)

1. Stop traffic to old server (load balancer drain).
2. `arcflow migrate up` with new binary/CLI from the release tag.
3. Deploy new server containers.
4. `curl /ready` until 200.
5. Restore traffic.

Rolling upgrade without dual-write: migrations must be backward compatible with the previous server version, or deploy in a stop-the-world window. ArcFlow additive migrations follow expand-only patterns where possible.

## Partial failure recovery
