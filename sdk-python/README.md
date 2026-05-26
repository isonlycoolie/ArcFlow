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

## Errors

| Exception | Meaning |
|-----------|---------|
| `WorkflowConfigurationError` | Fix the workflow before running (empty name, no steps, bad `step()` argument) |
| `WorkflowExecutionError` | A step failed during execution; check `run_id` / `failed_step` when set |

Messages use the format `[ArcFlow] <what happened>. <what to do>.`
