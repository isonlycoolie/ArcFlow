# ArcFlow contracts

Normative specifications and operator guides for production. Sprint verification, SDK design notes, and CI fixtures are not kept here.

## Normative (versioned contracts)

| Document | Purpose |
|----------|---------|
| [rcs-v1.schema.json](rcs-v1.schema.json) | Workflow and message data model |
| [TRACE-EVENT-SCHEMA-v1.md](TRACE-EVENT-SCHEMA-v1.md) | Trace event kinds and SEC-1 rules |
| [PROVIDER-API-CONTRACT-v1.md](PROVIDER-API-CONTRACT-v1.md) | LLM provider boundary |
| [RUNTIME-SERVER-API-v1.md](RUNTIME-SERVER-API-v1.md) | Self-hosted HTTP API |
| [CLI-SPEC-v1.md](CLI-SPEC-v1.md) | `arcflow` CLI commands |
| [recovery-schema-v1.sql](recovery-schema-v1.sql) | PostgreSQL DDL for workflow recovery |

Validate RCS schema: `bash scripts/validate-rcs-schema.sh`

## Guides

| Path | Audience |
|------|----------|
| [deployment/](deployment/) | Self-hosted runtime |
| [observability/](observability/) | Traces, events, debugging |
| [providers/](providers/) | LLM provider configuration |
| [cli/](cli/) | CLI usage |
| [typescript/](typescript/) | TypeScript SDK onboarding |
