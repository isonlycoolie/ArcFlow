# Debugging failed workflows with traces

Walkthrough: a two-step workflow where the second step fails.

## Setup

ArcFlow stub agents fail when `role="__fail__"` (`STUB_FAIL_ROLE` in Rust).

```python
from arcflow import Agent, Workflow
from arcflow.exceptions import WorkflowExecutionError
from arcflow._internal.runtime import get_trace

researcher = Agent(name="researcher", role="researcher", instructions="Research.")
failing = Agent(name="failing", role="__fail__", instructions="This step fails.")

wf = Workflow("debug-demo")
wf.step(researcher)
wf.step(failing)

try:
    wf.run("debug topic")
except WorkflowExecutionError as err:
    run_id = err.run_id
    trace = get_trace(run_id)  # run() raised before workflow.trace() is available
```

## Read the trace

```python
print(trace.status)           # failed
failed = trace.failed_step()
assert failed is not None
print(failed.step_index)      # 1 (second step)
print(failed.agent_name)      # failing
print(failed.error)           # StepError with error_code / message
```

## CLI check

```bash
arcflow trace <run-id> --format human --verbose
```

Expect event order ending with `StepFailed` then `WorkflowFailed`.

## What traces do not contain

- Workflow input (`"debug topic"`)
- Agent instructions
- Tool return values

If sensitive strings appear in trace JSON, treat it as a SEC-1 defect.
