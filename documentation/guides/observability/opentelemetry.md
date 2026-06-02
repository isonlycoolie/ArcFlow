
# OpenTelemetry integration

ArcFlow's primary observability path is the native execution trace (`TraceEventEmitter`, SDK `trace()`, HTTP trace, CLI). OpenTelemetry export is an **optional side channel** for platform teams that already run Grafana, Jaeger, or Prometheus.

OTel metrics and live span export are **alpha (OpenTelemetry metrics export)**. Behavior and label sets may change before production signoff. Core workflow correctness does not require OTel.

## ArcFlow-native first

| Path | Role |
|------|------|
| In-process trace events | Canonical source of truth |
| `GET /v1/runs/{id}/trace` | HTTP export for operators |
| OTLP export | Optional translation to OTel spans and metrics |

When OTel is disabled, overhead is near zero: no collector required for first workflow runs.

## Enable export

Set environment variables before starting the SDK host or `arcflow-server`:

| Variable | Default | Purpose |
|----------|---------|---------|
| `ARCFLOW_OTEL_ENABLED` | `false` | Master switch |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | unset | Collector URL, e.g. `http://localhost:4318` |
| `OTEL_EXPORTER_OTLP_PROTOCOL` | `http/protobuf` | `grpc` or `http/protobuf` |
| `OTEL_SERVICE_NAME` | `arcflow-runtime` | Resource attribute |
| `OTEL_RESOURCE_ATTRIBUTES` | unset | e.g. `deployment.environment=prod` |
| `ARCFLOW_OTLP_ENDPOINT` | unset | Legacy alias for endpoint |

Build `arcflow-core` with the `otel` feature when embedding the library directly:

```bash
cargo build -p arcflow-core --features otel
```

`arcflow-server` enables `otel` by default in its crate graph.

## Span hierarchy

```text
arcflow.workflow (run_id, workflow_name)
└── arcflow.step (step_id, step_index, agent_name)
 ├── arcflow.llm.invoke (provider, model, tokens.prompt, tokens.completion)
 ├── arcflow.tool.execute (tool_name, duration_ms, status)
 └── arcflow.memory (memory_type, operation)
```

Post-run OTLP export from `ExecutionTrace` remains as a fallback when live span export is unavailable. Both paths can be active when OTel is enabled.

Implementation: `runtime/arcflow-core/src/tracing/otel.rs`, `otel_metrics.rs`,.

## Metrics (OpenTelemetry export (alpha))

| Metric | Type | Labels |
|--------|------|--------|
| `arcflow.workflow.duration_ms` | Histogram | `status`, `workflow_name` |
| `arcflow.step.duration_ms` | Histogram | `step_id`, `status` |
| `arcflow.llm.tokens` | Counter | `provider`, `model`, `direction` |
| `arcflow.workflow.active` | UpDownCounter | (none) |
| `arcflow.retry.attempts` | Counter | `step_id` |
| `arcflow.recovery.resumes` | Counter | (none) |
| `arcflow.graph.iterations` | Counter | `node_id` |

Review label cardinality before enabling in high-tenant deployments. Prefer bounded label values (`workflow_name` from registry, not free-form user strings).

## Trace data policy on spans

Span attributes may include token counts, durations, ids, and status codes. They must **never** include prompt text, completion text, or raw provider bodies. The `otel_sec1` module and tests under `cargo test -p arcflow-core --features otel otel` encode this constraint.

Apply the same discipline as [Trace data policy rules](sec-1-rules.md) when adding custom instrumentation around ArcFlow.

## Local collector stack

Merge the OTel overlay with the server compose file:

```bash
docker compose -f docker/docker-compose.server.yml -f docker/docker-compose.otel.yml up
```

OTLP HTTP endpoint: `http://localhost:4318`. Jaeger UI: `http://localhost:16686`.

Run a workflow with:

```bash
export ARCFLOW_OTEL_ENABLED=true
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318
```

Expect `arcflow.workflow` spans within a few seconds of run completion.

See [OpenTelemetry observability](../../../docker/observability-otel.md) for full compose notes.

## OpenTelemetry metrics export maturity expectations

| Stable enough today | Still stabilizing under OpenTelemetry metrics export |
|---------------------|------------------------------|
| Native traces and HTTP trace API | Metric label sets |
| CLI TUI trace view | Dual live + post-run export tuning |
| metadata-only trace in span translation | Production SLO guidance for collector failures |

Export failures are best-effort and never fail workflow execution (`tracing/error.rs`). Monitor collector health separately.

## Verification commands

| Command | Expect |
|---------|--------|
| `cargo test -p arcflow-core --features otel otel` | Span, trace data policy, metrics smoke tests pass |
| `cargo build -p arcflow-core --no-default-features` | Pass without OTel deps |
| `cargo build -p arcflow-server` | Pass with OTel enabled |

## When not to use OTel yet

Skip OpenTelemetry metrics export in production if:

- You cannot cap metric cardinality.
- Compliance has not reviewed span attributes.
- You only need run-level debugging (native trace is sufficient).

Revisit when OpenTelemetry metrics export exits alpha in [maturity and known gaps](../../concepts/maturity-and-known-gaps.md).

## Related pages

- [Execution traces](execution-traces.md) for native trace access
- [Trace data policy rules](sec-1-rules.md) for attribute policy
- [Maturity and known gaps](../../concepts/maturity-and-known-gaps.md) for OpenTelemetry metrics export status
