# Self-hosted ArcFlow runtime

Run the HTTP server with `ARCFLOW_SERVER_API_KEY` set. Image: `arcflow/runtime` (build from `server/arcflow-server/Dockerfile`).

API contract: [server-api-v1.md](../../normative/runtime/server-api-v1.md). Recovery DDL: [recovery-schema-v1.sql](../../normative/runtime/recovery-schema-v1.sql).

Postgres for recovery and persistent memory: see `docker/docker-compose.dev.yml`.
