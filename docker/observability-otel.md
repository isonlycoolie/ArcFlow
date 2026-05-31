# OpenTelemetry observability for ArcFlow

ArcFlow exports workflow, step, LLM, tool, and memory spans to any OTLP collector when you opt in. The in-process trace store and CLI TUI stay unchanged, OTel is additive for platform teams running Grafana, Jaeger, or Prometheus alongside other services.

## Enable export

Set the master switch and a collector endpoint. Legacy `ARCFLOW_OTLP_ENDPOINT` still works; standard `OTEL_*` variables are preferred.

| Variable | Default | Purpose |
|----------|---------|---------|
| `ARCFLOW_OTEL_ENABLED` | `false` | Master switch, no outbound telemetry when unset |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | — | Collector URL, e.g. `http://localhost:4318` |
| `OTEL_EXPORTER_OTLP_PROTOCOL` | `http/protobuf` | `grpc` or `http/protobuf` |
| `OTEL_SERVICE_NAME` | `arcflow-runtime` | Resource attribute |
| `OTEL_RESOURCE_ATTRIBUTES` | — | e.g. `deployment.environment=prod` |

Build with the Cargo feature when running the core library directly:

```bash
cargo build -p arcflow-core --features otel
```

`arcflow-server` enables `otel` by default.

## Span hierarchy

```text
arcflow.workflow (run_id, workflow_name)
└── arcflow.step (step_id, step_index, agent_name)
    ├── arcflow.llm.invoke (provider, model, tokens.prompt, tokens.completion)
    ├── arcflow.tool.execute (tool_name, duration_ms, status)
    └── arcflow.memory (memory_type, operation)
```

Post-run OTLP export from `ExecutionTrace` remains as a fallback. Both paths can be active when OTel is enabled.

## Metrics (alpha)

| Metric | Type | Labels |
|--------|------|--------|
| `arcflow.workflow.duration_ms` | Histogram | `status`, `workflow_name` |
| `arcflow.step.duration_ms` | Histogram | `step_id`, `status` |
| `arcflow.llm.tokens` | Counter | `provider`, `model`, `direction` |
| `arcflow.workflow.active` | UpDownCounter | — |
| `arcflow.retry.attempts` | Counter | `step_id` |
| `arcflow.recovery.resumes` | Counter | — |
| `arcflow.graph.iterations` | Counter | `node_id` |

## SEC-1 on spans

Span attributes may include token counts and metadata (`step_id`, `status`, `duration_ms`). They must never include prompt text, completion text, or raw provider bodies. The `otel_sec1` module and `cargo test -p arcflow-core --features otel otel` encode this constraint.

## Local collector

Merge the OTel overlay with the server stack:

```bash
docker compose -f docker/docker-compose.server.yml -f docker/docker-compose.otel.yml up
```

Open Jaeger at http://localhost:16686 and run a workflow with `ARCFLOW_OTEL_ENABLED=true`. You should see `arcflow.workflow` spans within a few seconds.

## Verification

| Command | Expect |
|---------|--------|
| `cargo test -p arcflow-core --features otel otel` | Span, SEC-1, and metrics smoke tests pass |
| `cargo build -p arcflow-core --no-default-features` | Pass without OTel deps |
| `cargo build -p arcflow-server` | Pass with OTel enabled |
