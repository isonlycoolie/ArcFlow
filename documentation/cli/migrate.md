**Audience:** `[operator]` `[platform]`

# arcflow migrate up

Apply embedded PostgreSQL schema migrations shipped in `runtime/arcflow-core/migrations/`. Required before `arcflow-server` can serve `POST /v1/runs` on a fresh database.

## Usage

```bash
export ARCFLOW_POSTGRESQL_URL=postgres://arcflow:arcflow@localhost:5432/arcflow
arcflow migrate up
```

Subcommand structure:

```bash
arcflow migrate up
```

(No additional flags today.)

## Expected output

Success:

```text
[ArcFlow] migrations applied.
```

Failure examples:

```text
[ArcFlow] ARCFLOW_POSTGRESQL_URL must be set for migrate up.
[ArcFlow] postgres connect failed: ...
[ArcFlow] migration failed: ...
```

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | All pending migrations applied |
| 1 | Connect or migration SQL error |
| 2 | Missing `ARCFLOW_POSTGRESQL_URL` |

## Idempotent behavior

Migrations use `IF NOT EXISTS` and version tracking inside `arcflow_core::migrate::run`. Re-running `migrate up` on an up-to-date database succeeds without changes.

## Seven migrations

| Version | Creates / alters |
|---------|------------------|
| 000001 | `arcflow_recovery_state` |
| 000002 | Graph columns on recovery |
| 000003 | `arcflow_runs` |
| 000004 | `arcflow_human_approvals`, run snapshots |
| 000005 | `arcflow_trace_events` |
| 000006 | `arcflow_workflows`, `arcflow_workflow_aliases` |
| 000007 | `arcflow_sites`, tokens, usage daily |

Table detail: [server/postgres-schema.md](../server/postgres-schema.md).

## Deploy runbook

**Option A: CLI before server rollout**

```bash
export ARCFLOW_POSTGRESQL_URL=...
arcflow migrate up
# deploy arcflow-server
curl -sf https://api.example.com/ready
```

**Option B: Docker Compose init container**

`docker/docker-compose.server.yml` runs `arcflow-migrate` once Postgres is healthy, then starts the server.

**When to prefer manual migrate**

- Blue/green deploys where DB migrates in a separate job
- CI gates that fail deploy if schema drift detected
- Managed Postgres where init containers are not used

## Verify after migrate

```bash
curl -s -o /dev/null -w "%{http_code}\n" http://localhost:8080/ready
```

Expect **200**. **503** indicates connection failure or pending migrations.

## Related pages

- [server/postgres-schema.md](../server/postgres-schema.md)
- [server/overview.md](../server/overview.md)
- [cli/overview.md](overview.md)

**Source:** capabilities reference §18, Appendix G; `cli/arcflow-cli/src/commands/migrate.rs`, `runtime/arcflow-core/src/migrate.rs`.
