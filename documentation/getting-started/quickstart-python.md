# Python quickstart


## Before you start

If you only need the smallest runnable example, start with [First workflow in five minutes](first-workflow-in-five-minutes.md). This page adds install verification, trace reading, optional OpenAI, error handling, and remote server mode in one sitting.

Read [03 Anatomy of a workflow](fundamentals/03-anatomy-of-a-workflow.md) so terms like step, run input, and `result.output` are already familiar.

## Concept

The Python SDK exposes idiomatic `Agent` and `Workflow` types that serialize to the Runtime Contract Specification (RCS) and call the in-process Rust runtime by default. You declare agents and step order in Python; `arcflow-core` executes the graph, invokes providers, and emits trace events.

Without a `provider=` argument, `run()` uses the default in-process agent backend. No `OPENAI_API_KEY` is required on that path. Passing `runtime="http://localhost:8080"` targets `arcflow-server` instead while keeping the same declaration code.

## Install

From the repository root:

```bash
cd sdk-python
pip install maturin
maturin develop
pip install -e ".[dev]"
```

Confirm the package loads:

```bash
python -c "from arcflow import Agent, Workflow; print('import ok')"
```

See [Install and build](install-and-build.md) for platform notes (macOS, Linux, Windows) and troubleshooting.

## Example: agent and workflow

Save as `quickstart.py`:

```python
from arcflow import Agent, Workflow

researcher = Agent(
    name="researcher",
    role="research",
    instructions="Research the given topic and list key facts.",
)
writer = Agent(
    name="writer",
    role="write",
    instructions="Turn the research into a short paragraph.",
)

workflow = Workflow("research_pipeline")
workflow.step(researcher)
workflow.step(writer)

result = workflow.run("Analyze renewable energy trends")

print(result.output)
print(result.run_id)
print(result.step_count)
print(result.status)
```

Run:

```bash
python quickstart.py
```

You should see `step_count == 2`, `status` of `completed`, and non-empty `output`.

## Reading the result

`WorkflowResult` fields used most often:

| Field | Meaning |
|-------|---------|
| `output` | Text from the final step |
| `run_id` | UUID for this run; use with `workflow.trace()` or CLI |
| `step_count` | Number of steps executed |
| `status` | Terminal status string (typically `completed`) |
| `trace_events` | Tuple of metadata-only trace event dicts |

```python
for event in result.trace_events:
    print(event.get("event_kind"), event.get("sequence"))
```

Trace payloads follow SEC-1 rules: no raw prompts, tool values, or credentials. See [SEC-1 rules](../guides/observability/sec-1-rules.md) and [Trace events (normative)](../contracts/trace-events-normative.md) for event shapes.

## Structured trace via `workflow.trace()`

After `run()`, the workflow object exposes a parsed trace:

```python
trace = workflow.trace()
print(trace.summary())
print(trace.status)
print(len(trace.steps))
print(trace.total_tokens_consumed)
```

Calling `trace()` before `run()` raises `TraceNotFoundError`.

## Optional: real LLM with OpenAI

When you have an API key, pass a provider to `run()`:

```python
import os

from arcflow import Agent, OpenAI, Workflow

os.environ.setdefault("OPENAI_API_KEY", "sk-your-key-here")

wf = Workflow("demo")
wf.step(Agent(name="writer", role="author", instructions="Summarize in three sentences."))

result = wf.run(
    "Quantum networking",
    provider=OpenAI(model="gpt-4o"),
)
print(result.output)
```

Supported environment variables for other providers: `ANTHROPIC_API_KEY`, `GEMINI_API_KEY` with `Anthropic` and `Gemini` classes from `arcflow`.

Provider failures raise `ProviderExecutionError` with a `provider_id` when set. Configuration mistakes raise `ProviderConfigurationError` before the run starts.

See [Provider configuration](../guides/agents-and-tools/provider-configuration.md) for model params and retry behavior.

## Remote server mode (optional)

To target `arcflow-server` instead of the in-process runtime:

```python
import os

os.environ["ARCFLOW_SERVER_API_KEY"] = "dev-secret"

from arcflow import Agent, Workflow

wf = Workflow("demo", runtime="http://localhost:8080")
wf.step(Agent(name="writer", role="author", instructions="Summarize."))
result = wf.run("hello")
print(result.output)
```

See [Server API quickstart](quickstart-server-api.md) and [Integrating track](integrating/README.md).

## Common errors

| Exception | Typical cause |
|-----------|----------------|
| `WorkflowConfigurationError` | Empty workflow name, no steps, invalid `step()` argument |
| `WorkflowExecutionError` | Step failed at runtime; inspect `run_id` / `failed_step` on the exception |
| `TraceNotFoundError` | `trace()` called before `run()` or run evicted from store |

Messages use the format `[ArcFlow] <what happened>. <what to do>.`

## Verify

| Check | Expected |
|-------|----------|
| Import succeeds | `import ok` |
| Default run (no API key) | `step_count == 2`, non-empty `output` |
| `workflow.trace()` after run | Summary prints without exception |
| Server mode (optional) | Same output shape when stack is up |

## Next

| Topic | Link |
|-------|------|
| Guided first workflow with verification | [Track A: First workflow](../tutorials/track-a-first-workflow.md) |
| Linear workflow design | [Linear workflows](../guides/workflows/linear-workflows.md) |
| Provider configuration detail | [Provider configuration](../guides/agents-and-tools/provider-configuration.md) |
| Execution traces | [Execution traces](../guides/observability/execution-traces.md) |
| TypeScript twin | [TypeScript quickstart](quickstart-typescript.md) |
| Server HTTP detail | [Server API quickstart](quickstart-server-api.md) |
