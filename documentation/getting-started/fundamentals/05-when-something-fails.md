# When something fails

**Audience:** `[developer]`

## Before you start

Work through [Default runtime vs live LLM](04-stub-vs-live-provider.md) so you have seen both successful default runs and (optionally) live provider setup. Keep the SDK installed per [Install and build](../install-and-build.md).

## Concept

ArcFlow splits failures into two phases. The phase tells you whether to edit your workflow file or inspect a specific run.

**`WorkflowConfigurationError`** means the workflow or agent definition is invalid, or you called APIs in the wrong order. Nothing useful reached the engine, or validation stopped the run before execution began. Examples: empty agent name, no steps registered, empty run input, passing a string to `step()` instead of an `Agent`.

**`WorkflowExecutionError`** means execution started and a step failed at runtime. The definition was structurally acceptable, but something went wrong inside the engine or provider layer. Examples: live provider HTTP failure after retries, tool execution failure, timeout. Subclasses like `RetryExhaustedError` and `WorkflowTimeoutError` add attributes; the base class carries `run_id` and `failed_step` when the runtime knows them.

Reading the message: ArcFlow SDK validation uses a two-part sentence:

```
[ArcFlow] <what happened>. <what to do>.
```

The `[ArcFlow]` prefix lets you grep logs. The first clause is the rule you broke. The second clause is the fix. Configuration errors never include a `run_id` because there is no run to inspect. Execution errors often do; use `err.run_id` and `workflow.trace()` after catching them.

| Signal | Exception | Typical fix |
|--------|-----------|-------------|
| Empty `Agent.name` | `WorkflowConfigurationError` | Provide a non-empty string |
| `run()` with no steps | `WorkflowConfigurationError` | Call `step()` at least once |
| `run("")` | `WorkflowConfigurationError` | Pass non-empty input |
| Live provider outage mid-run | `WorkflowExecutionError` or `ProviderExecutionError` | Check provider status, retry policy, trace |
| `trace()` before `run()` | `TraceNotFoundError` | Run the workflow first |

For a full exception hierarchy see [Python SDK exception reference](../../sdks/python/exception-reference.md). This lesson stays with the two types you will see most often in the first week.

## Minimal example

Save as `failure_modes.py`:

```python
from arcflow import Agent, Workflow
from arcflow.exceptions import WorkflowConfigurationError

# Configuration error: invalid agent
try:
    Agent(name="", role="x", instructions="y")
except WorkflowConfigurationError as err:
    print("config (agent):", err)

# Configuration error: no steps
try:
    Workflow("lonely").run("hello")
except WorkflowConfigurationError as err:
    print("config (workflow):", err)

# Success path for contrast
ok = Agent(name="writer", role="write", instructions="Reply briefly.")
wf = Workflow("ok")
wf.step(ok)
result = wf.run("test input")
print("success run_id:", result.run_id)
```

Run:

```bash
python failure_modes.py
```

You should see two `[ArcFlow]` configuration messages, then a successful UUID on the last line.

## Verify

**Spot the phase.** For each message you print, ask: "Did a run start?" If no `run_id` exists and the error mentions `must be` or `Cannot run`, it is configuration. Fix the definition and rerun.

**Execution error shape (optional read).** Live provider failures require a key and network. If you run a workflow with `provider=OpenAI(model="gpt-4o")` and an invalid key, you may see `ProviderExecutionError` (a related execution-time type). Inspect:

```python
except ProviderExecutionError as err:
    print(err)
    print(getattr(err, "provider_id", None))
```

Stub-only workflows in this track rarely raise `WorkflowExecutionError` unless you add tools, timeouts, or test hooks later.

**Trace after execution failure.** When `WorkflowExecutionError` includes `run_id`, fetch detail:

```python
except WorkflowExecutionError as err:
    print(err.run_id, err.failed_step)
    # after run() on the same workflow instance:
    # trace = workflow.trace()
```

**Message format drill.** Pick any `WorkflowConfigurationError` from this track (empty name, no steps). Confirm both sentences are present: problem, then remediation.

## Next steps

You have finished the fundamentals track. Continue with:

| Goal | Link |
|------|------|
| Shortest end-to-end example | [First workflow in five minutes](../first-workflow-in-five-minutes.md) |
| OpenAI, traces, remote runtime | [Python quickstart](../quickstart-python.md) |
| Step-by-step verification lab | [Track A: First workflow](../../tutorials/track-a-first-workflow.md) |
| Full exception list | [Python SDK exception reference](../../sdks/python/exception-reference.md) |

Return to the [Fundamentals track overview](README.md) if you want the lesson index.

**Source:** `sdk-python/arcflow/exceptions.py`, `sdk-python/arcflow/agent.py`, `sdk-python/arcflow/workflow.py`, `documentation/sdks/python/exception-reference.md`; capabilities reference §16.2.
