# Anatomy of a workflow

**Audience:** `[developer]`

## Before you start

You should understand agents from [Anatomy of an agent](02-anatomy-of-an-agent.md). Confirm the SDK is installed ([Install and build](../install-and-build.md)).

## Concept

A `Workflow` is an ordered pipeline of agents. You create it, register steps, then call `run()` with a string input. The workflow name is a label for traces and debugging; it defaults to `"default"` if you omit it, but giving a descriptive name helps when you inspect logs later.

Typical lifecycle:

1. `Workflow("my_pipeline")` creates an empty linear workflow.
2. `workflow.step(agent)` appends an agent in order. Each call returns the same workflow object, so you can chain calls.
3. `workflow.run("user input")` validates, serializes to RCS, and executes in Rust.
4. `run()` returns a `WorkflowResult` dataclass with the fields you read most often in application code.

`WorkflowResult` fields for beginner workflows:

| Field | Type | Meaning |
|-------|------|---------|
| `output` | `str` | Text from the final step. This is what you usually show to the user. |
| `run_id` | `str` | UUID for this execution. Use it with `workflow.trace()` or CLI trace commands. |
| `step_count` | `int` | How many steps ran (equals the number of `step()` calls in a simple linear workflow). |
| `status` | `str` | Terminal status string, typically `"completed"` on success. |
| `trace_events` | `tuple[dict, ...]` | Metadata-only trace events for this run (no raw prompts in SEC-1 safe payloads). |
| `approval_key` | `str \| None` | Set when human-in-the-loop interrupts a run; `None` for normal completed runs. |

Validation on `run()` catches common mistakes early: empty input, or a workflow with no steps. Those raise `WorkflowConfigurationError` with messages like `Cannot run a workflow with no steps` or `Workflow input must be a non-empty string`.

`step()` also validates its argument. Passing a string instead of an `Agent` raises `WorkflowConfigurationError` explaining that `step()` requires an `Agent` instance.

## Minimal example

Save as `workflow_anatomy.py`:

```python
from arcflow import Agent, Workflow

researcher = Agent(
    name="researcher",
    role="research",
    instructions="List three facts about the topic.",
)
writer = Agent(
    name="writer",
    role="write",
    instructions="Turn the facts into one paragraph.",
)

workflow = Workflow("research_pipeline")
workflow.step(researcher).step(writer)

result = workflow.run("Wind energy in 2026")

print("output:", result.output[:120], "..." if len(result.output) > 120 else "")
print("run_id:", result.run_id)
print("step_count:", result.step_count)
print("status:", result.status)
print("trace_event_count:", len(result.trace_events))
```

Run:

```bash
python workflow_anatomy.py
```

## Verify

**Happy path.** Expect `step_count == 2`, `status == "completed"`, non-empty `output`, and a UUID-shaped `run_id`.

**No steps.** Comment out both `step()` lines and wrap `run()` in a try/except:

```python
from arcflow.exceptions import WorkflowConfigurationError

try:
    Workflow("empty").run("hello")
except WorkflowConfigurationError as err:
    print(err)
```

You should see `[ArcFlow] Cannot run a workflow with no steps.`

**Empty input.** With at least one step registered, call `run("")` or `run("   ")`. Expect `[ArcFlow] Workflow input must be a non-empty string.`

**Trace after run.** After a successful run, `workflow.trace()` returns a parsed trace object. Calling it before `run()` raises `TraceNotFoundError`. That behavior is covered in the [Python quickstart](../quickstart-python.md); you do not need it for this lesson beyond knowing it exists.

## Next lesson

[Default runtime vs live LLM](04-stub-vs-live-provider.md): why no API key is required by default and how to opt into OpenAI when you are ready.

**Source:** `sdk-python/arcflow/workflow.py`, `sdk-python/arcflow/result.py`, `sdk-python/tests/integration/test_first_five_minutes.py`; capabilities reference §16.2.
