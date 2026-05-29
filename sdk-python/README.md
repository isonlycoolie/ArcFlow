# ArcFlow Python SDK

Build AI workflows as ordered pipelines of agents. Execution runs in the ArcFlow Rust runtime; Python defines structure only.

## Install (development)

```bash
cd sdk-python
pip install maturin
maturin develop
pip install -e ".[dev]"
```

## Quick start

```python
from arcflow import Agent, Workflow

researcher = Agent(
    name="researcher",
    role="research",
    instructions="Research the given topic thoroughly.",
)
writer = Agent(
    name="writer",
    role="write",
    instructions="Write a clear summary of the research.",
)

workflow = Workflow()
workflow.step(researcher)
workflow.step(writer)

result = workflow.run("Analyze renewable energy trends")
print(result.output)
```

## What just happened

You declared two agents and chained them into a workflow. `run()` sent the definition and input to the runtime, which executed steps in order using deterministic stub agents (Sprint 2–3). The last step’s text is exposed as `result.output`.

## Requirements

- Python 3.9+
- Rust toolchain (for building the extension from source)
- No runtime Python dependencies

## Tools (Sprint 4)

```python
from arcflow import Agent, Tool, Workflow

def search(payload: dict) -> str:
    return f"results-for:{payload.get('message', '')}"

tool = Tool(
    name="search",
    description="Search the web",
    input_schema={
        "type": "object",
        "properties": {"message": {"type": "string"}},
    },
    execute=search,
)
agent = Agent(
    name="researcher",
    role="researcher",
    instructions="Use tools when needed.",
    tools=(tool,),
)
Workflow("demo").step(agent).run("query text")
```

Tool inputs are validated in Rust (`jsonschema`). Execution stays in the native runtime; Python supplies callables only.

`WorkflowResult.trace_events` exposes metadata-only RCS trace events (`ToolExecuted`, `MemoryRead`, `MemoryWrite`, workflow lifecycle). Payload values are never included.

## Memory (Sprint 4)

```python
from arcflow import Agent, MemoryConfig, MemoryScope, MemoryType

agent = Agent(
    name="a",
    role="researcher",
    instructions="Remember context.",
    memory=MemoryConfig(MemoryType.SESSION, MemoryScope.AGENT),
)
```

| Type | Backend | Env var |
|------|---------|---------|
| Session / Shared | In-process | — |
| Persistent | PostgreSQL | `ARCFLOW_POSTGRESQL_URL` |
| Vector | Qdrant | `ARCFLOW_QDRANT_URL` |

Local Docker stack:

```bash
docker compose -f docker/docker-compose.dev.yml up -d
export ARCFLOW_POSTGRESQL_URL=postgresql://arcflow:arcflow@localhost:5432/arcflow
export ARCFLOW_QDRANT_URL=http://localhost:6333
```

## Errors

| Exception | Meaning |
|-----------|---------|
| `WorkflowConfigurationError` | Fix the workflow before running (empty name, no steps, bad `step()` argument) |
| `WorkflowExecutionError` | A step failed during execution; check `run_id` / `failed_step` when set |
| `ToolConfigurationError` | Invalid tool definition before run |
| `ToolExecutionError` | Tool failed during a run |
| `MemoryConfigurationError` | Invalid memory config on an agent |
| `MemoryOperationError` | Memory read/write failed |
| `InfrastructureUnavailableError` | Postgres/Qdrant unreachable or URL unset |
| `TraceNotFoundError` | No trace for last run (call `run()` first) or run evicted from store |
| `TraceStorageWarning` | Trace store dropped events at capacity |
| `ProviderConfigurationError` | Invalid provider config before run |
| `ProviderExecutionError` | LLM provider call failed; check `provider_id` |

Messages use the format `[ArcFlow] <what happened>. <what to do>.`

## Observability (Sprint 5)

```python
from arcflow import Agent, Workflow

wf = Workflow("demo")
wf.step(Agent(name="researcher", role="researcher", instructions="Research."))
result = wf.run("Analyze renewable energy trends")
trace = wf.trace()
print(trace.summary())
print(trace.status)
print(trace.failed_step())
```

```bash
# From repo root (in-process store; same process as the SDK run)
cargo run -p arcflow-cli -- trace <run-id> --format json --verbose
```

Optional OTLP export: set `ARCFLOW_OTLP_ENDPOINT` before running workflows.

See [contracts/guides/observability/](../contracts/guides/observability/) and [trace-events-v1.md](../contracts/normative/observability/trace-events-v1.md).

## Providers (Sprint 6)

```python
from arcflow import Agent, OpenAI, Workflow

wf = Workflow("demo")
wf.step(Agent(name="writer", role="author", instructions="Summarize."))
result = wf.run("topic", provider=OpenAI(model="gpt-4o"))
```

Set `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, or `GEMINI_API_KEY` in the environment. Without `provider=`, workflows use the stub agent (backwards compatible).

See [contracts/guides/providers/getting-started.md](../contracts/guides/providers/getting-started.md).
