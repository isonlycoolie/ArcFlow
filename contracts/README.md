# ArcFlow contracts

Production specifications and operator guides. Normative documents define wire formats and runtime boundaries; guides explain how to configure and operate ArcFlow.

```
contracts/
├── README.md
├── normative/          # Versioned contracts — breaking changes bump version
│   ├── rcs/            # Data model (workflows, agents, messages)
│   ├── observability/  # Trace event schema
│   ├── providers/      # LLM provider API boundary
│   ├── runtime/        # HTTP server API and recovery DDL
│   └── cli/            # CLI command surface
└── guides/             # Operator and integrator documentation
    ├── deployment/
    ├── observability/
    ├── providers/
    ├── cli/
    └── sdks/
```

## Normative

See [normative/README.md](normative/README.md) for the full index.

| Document | Purpose |
|----------|---------|
| [normative/rcs/v1.schema.json](normative/rcs/v1.schema.json) | Workflow and message data model (RCS) |
| [normative/observability/trace-events-v1.md](normative/observability/trace-events-v1.md) | Trace event kinds and SEC-1 rules |
| [normative/providers/api-v1.md](normative/providers/api-v1.md) | LLM provider boundary |
| [normative/runtime/server-api-v1.md](normative/runtime/server-api-v1.md) | Self-hosted HTTP API |
| [normative/runtime/recovery-schema-v1.sql](normative/runtime/recovery-schema-v1.sql) | PostgreSQL DDL for workflow recovery |
| [normative/cli/spec-v1.md](normative/cli/spec-v1.md) | `arcflow` CLI commands |

Validate RCS schema: `bash scripts/validate-rcs-schema.sh`

## Guides

See [guides/README.md](guides/README.md) for the full index.

| Path | Audience |
|------|----------|
| [guides/deployment/](guides/deployment/) | Self-hosted runtime |
| [guides/observability/](guides/observability/) | Traces, events, debugging |
| [guides/providers/](guides/providers/) | LLM provider configuration |
| [guides/cli/](guides/cli/) | CLI usage |
| [guides/sdks/typescript/](guides/sdks/typescript/) | TypeScript SDK onboarding |
