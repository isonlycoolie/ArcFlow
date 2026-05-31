# First workflow in five minutes

**Audience:** `[developer]`

## Before you start

Complete the [Python install](install-and-build.md#python-sdk-maturin--pip) section (`maturin develop` in `sdk-python/`). The example below is Python. The TypeScript equivalent lives in [TypeScript quickstart](quickstart-typescript.md).

You do not need an LLM API key, Docker, or a running server for this path.

## Concept

This is the shortest path to a working ArcFlow run. You define two agents, chain them in order, call `run()`, and read the final output. With no `provider=` argument, the runtime uses its built-in default agent backend, which returns deterministic placeholder text so you can validate wiring before connecting OpenAI or another model.

An `Agent` is a named role with instructions. A `Workflow` is an ordered list of agents executed by the Rust runtime. Python declares structure; it does not execute LLM calls itself.

## Example

Save as `first_workflow.py` anywhere on your machine (with the `arcflow` package importable from your virtual environment):

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

workflow = Workflow("research_pipeline")
workflow.step(researcher)
workflow.step(writer)

result = workflow.run("Analyze renewable energy trends")
print(result.output)
print(f"run_id={result.run_id} steps={result.step_count} status={result.status}")
```

Run it:

```bash
python first_workflow.py
```

## Verify

| Check | Expected |
|-------|----------|
| Script exits without exception | Yes |
| First `print` line | Non-empty text |
| `result.step_count` | `2` |
| `result.status` | `completed` |
| `result.run_id` | UUID string |

Exact output text may vary by runtime version. The integration test asserts `len(result.output) > 0` and `result.step_count == 2`.

This matches the canonical example in `sdk-python/tests/integration/test_first_five_minutes.py`.

## What just happened

You declared two agents and registered them as ordered steps on a `Workflow`. Calling `run()` serialized the definition and input to the ArcFlow Rust runtime, which executed step one then step two. The last step's output surface is `result.output`. Trace metadata is available on `result.trace_events` and via `workflow.trace()` after the run.

## Next

| Track | Document |
|-------|----------|
| Full curriculum | [Getting started README](README.md) |
| Guided verification (trace events, status checks) | [Track A: First workflow](../tutorials/track-a-first-workflow.md) |
| OpenAI and structured traces | [Python quickstart](quickstart-python.md) |
| TypeScript twin | [TypeScript quickstart](quickstart-typescript.md) |
| HTTP instead of embedded SDK | [Server API quickstart](quickstart-server-api.md) |
| Integration choices | [Integrating track](integrating/README.md) |

## Source

`sdk-python/README.md`, `sdk-python/tests/integration/test_first_five_minutes.py`; capabilities reference §16.2, §28 Track A.
