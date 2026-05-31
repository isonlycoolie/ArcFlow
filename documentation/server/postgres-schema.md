**Audience:** `[platform]`

# Postgres schema

ArcFlow stores server-side state in PostgreSQL. Migrations live in `runtime/arcflow-core/migrations/` and apply via `arcflow migrate up` or the `arcflow-migrate` init container before server start.

Verify readiness after migrate:

```bash
curl -s -o /dev/null -w "%{http_code}\n" http://localhost:8080/ready
```

## Migration index

| File | Purpose |
|------|---------|
| `20240531000001_recovery_state.sql` | Linear recovery rows |
| `20240531000002_recovery_graph_columns.sql` | Graph checkpoint columns on recovery |
| `20240531000003_arcflow_runs.sql` | Run tracking and idempotency |
| `20240531000004_human_approvals.sql` | HITL approvals + run snapshots |
| `20240531000005_trace_events.sql` | Persisted SEC-1 trace events |
| `20240531000006_workflow_registry.sql` | Semver workflow registry |
| `20240531000007_arcflow_sites.sql` | Static product sites and tokens |

Apply manually:

```bash
export ARCFLOW_POSTGRESQL_URL=postgres://arcflow:arcflow@localhost:5432/arcflow
cargo run -p arcflow-cli -- migrate up
```

## arcflow_recovery_state

Supports partial execution recovery (Sprint 7).

| Column | Type | Notes |
|--------|------|-------|
| `recovery_id` | TEXT PK | Recovery record id |
| `original_run_id` | TEXT UNIQUE | Run being recovered |
| `workflow_def_id` | TEXT | Workflow definition id |
| `state_json` | JSONB | Serialized recovery payload |
| `created_at` | TIMESTAMPTZ | Insert time |
| `is_consumed` | BOOLEAN | Prevents double resume |
| `execution_mode` | TEXT | `linear` or `graph` (migration 000002) |
| `current_node_id` | TEXT | Graph checkpoint (FP-1.01 partial) |
| `graph_iteration_count` | INTEGER | Graph guard |
| `pending_join` | JSONB | Join node state |

Indexes: `original_run_id`, unconsumed by workflow.

## arcflow_runs

HTTP run lifecycle and idempotency.

| Column | Type | Notes |
|--------|------|-------|
| `run_id` | TEXT PK | UUID string |
| `trace_id` | TEXT | Trace correlation |
| `status` | TEXT | ExecutionStatus PascalCase |
| `workflow_hash` | TEXT | Workflow id fingerprint |
| `exec_config_json` | JSONB | Parsed exec_config |
| `result_json` | JSONB | Terminal success payload |
| `error_json` | JSONB | Terminal error |
| `idempotency_key` | TEXT UNIQUE | Optional dedup key |
| `workflow_json` | JSONB | Snapshot (migration 000004) |
| `agents_json` | JSONB | Agent snapshot |
| `input_text` | TEXT | Run input |
| `created_at`, `started_at`, `completed_at` | TIMESTAMPTZ | Timestamps |

Indexes: `status`, `created_at DESC`.

## arcflow_human_approvals

HITL approval records.

| Column | Type | Notes |
|--------|------|-------|
| `run_id`, `approval_key` | TEXT | Composite PK |
| `status` | TEXT | pending / resolved |
| `approved` | BOOLEAN | Decision |
| `data_json` | JSONB | Approver payload |
| `expires_at` | TIMESTAMPTZ | HumanTimeout boundary |
| `created_at`, `resolved_at` | TIMESTAMPTZ | Audit |

Partial index on pending approvals by expiry.

## arcflow_trace_events

Persisted trace for `GET /v1/runs/{id}/trace`.

| Column | Type | Notes |
|--------|------|-------|
| `run_id`, `seq` | TEXT, BIGINT | Composite PK, ordered |
| `event_json` | JSONB | SEC-1 metadata event |
| `created_at` | TIMESTAMPTZ | Insert time |

## arcflow_workflows and arcflow_workflow_aliases

Registry semver storage.

**arcflow_workflows**

| Column | Type | Notes |
|--------|------|-------|
| `id` | UUID PK | Row id |
| `name`, `version` | TEXT | UNIQUE pair |
| `schema_hash` | TEXT | Definition hash |
| `definition_json` | JSONB | Full workflow |
| `published_by` | TEXT | Optional publisher |
| `published_at` | TIMESTAMPTZ | Publish time |
| `deprecated` | BOOLEAN | Soft deprecation |

**arcflow_workflow_aliases**

| Column | Type | Notes |
|--------|------|-------|
| `name`, `alias` | TEXT | Composite PK |
| `version` | TEXT | Target version |

Requires `pgcrypto` extension (migration 000006).

## arcflow_sites, arcflow_site_tokens, arcflow_site_usage_daily

Static product operator model.

**arcflow_sites**

| Column | Type | Notes |
|--------|------|-------|
| `id` | TEXT PK | Site id |
| `display_name` | TEXT | Operator label |
| `allowed_origins` | TEXT[] | Relay Origin allowlist |
| `rate_limit_rpm` | INTEGER | Default 60 |
| `allow_inline` | BOOLEAN | Browser inline workflow override |
| `default_workflow_name` | TEXT | e.g. `chat` |
| `kb_namespace` | TEXT | Qdrant namespace |
| `upstream_runtime_key` | TEXT | Scoped server key id |
| `chat_instructions` | TEXT | Publish template hint |

**arcflow_site_tokens**

Hashed tokens per site; `revoked_at` for rotation.

**arcflow_site_usage_daily**

Daily run counters per site for operator metrics.

## Operational notes

- Relay does **not** connect to Postgres; it reads site config from env JSON or future sync.
- Qdrant collections for vector memory are **not** in these migrations; configure `ARCFLOW_QDRANT_URL` separately.
- Backup `arcflow_runs` and `arcflow_trace_events` together for audit replay.

## Related pages

- [cli/migrate.md](../cli/migrate.md)
- [server/overview.md](overview.md)
- [static-product/site-lifecycle.md](../static-product/site-lifecycle.md)

**Source:** capabilities reference Appendix G, §12.2; `runtime/arcflow-core/migrations/`; K-04.
