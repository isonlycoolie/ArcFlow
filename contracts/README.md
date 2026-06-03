# ArcFlow contracts

Versioned wire formats and runtime boundaries. JSON Schema `$id` URIs use **https://arcflows.vercel.com** as the canonical host until **arcflow.dev** is live. Integrator-facing narrative documentation lives under [documentation/](../documentation/README.md).

```
contracts/
├── README.md
└── normative/          # Versioned contracts — breaking changes bump version
    ├── rcs/            # Data model (workflows, agents, messages)
    ├── observability/  # Trace event schema
    ├── providers/      # LLM provider API boundary
    ├── runtime/        # Recovery DDL
    └── cli/            # CLI command surface
```

## Normative

See [normative/README.md](normative/README.md) for the full index.

| Document | Purpose |
|----------|---------|
| [normative/rcs/v1.schema.json](normative/rcs/v1.schema.json) | Workflow and message data model (RCS) |
| [normative/observability/trace-events-v1.md](normative/observability/trace-events-v1.md) | Trace event kinds and trace data policy |
| [normative/providers/api-v1.md](normative/providers/api-v1.md) | LLM provider boundary |
| [normative/runtime/recovery-schema-v1.sql](normative/runtime/recovery-schema-v1.sql) | PostgreSQL DDL for workflow recovery |
| [normative/cli/spec-v1.md](normative/cli/spec-v1.md) | `arcflow` CLI commands |

HTTP server routes (integrator reference): [documentation/server/http-api-reference.md](../documentation/server/http-api-reference.md)

Validate RCS schema: `bash scripts/validate-rcs-schema.sh`

## Documentation

| Topic | Start here |
|-------|------------|
| Deployment | [documentation/deployment/overview.md](../documentation/deployment/overview.md) |
| Observability | [documentation/guides/observability/execution-traces.md](../documentation/guides/observability/execution-traces.md) |
| Providers | [documentation/guides/agents-and-tools/provider-configuration.md](../documentation/guides/agents-and-tools/provider-configuration.md) |
| CLI | [documentation/cli/overview.md](../documentation/cli/overview.md) |
| TypeScript SDK | [documentation/getting-started/quickstart-typescript.md](../documentation/getting-started/quickstart-typescript.md) |
