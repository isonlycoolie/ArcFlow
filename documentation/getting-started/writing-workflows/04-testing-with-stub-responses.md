# 04 Testing with stub responses


## Before you start

Complete [01 Linear pipelines](01-linear-pipelines.md). You should know how to register steps and read `result.output`. No pytest knowledge is required for the first example, though the Verify section mentions optional pytest markers.

## Concept

`workflow.test(cases)` runs your workflow in **test mode**. The Rust runtime returns deterministic step outputs from a `stub_responses` map instead of calling a live provider. This is how you assert pipeline shape in CI without API keys or network access.

Each case is a dict:

| Field | Purpose |
|-------|---------|
| `name` | Label for the result row (optional) |
| `input` | Run input string |
| `expected_output` | If set, pass/fail compares final output to this string |
| `stub_responses` | Map of step keys to mock behavior |

Stub keys follow step order: `step_1`, `step_2`, `step_3`, and so on (not agent names).

Each stub entry can include:

| Stub field | Behavior |
|------------|----------|
| `output` | Text returned when the step succeeds |
| `fail_times` | How many attempts fail before success |
| `then_output` | Output after failures are exhausted |

When you set `expected_output` without `stub_responses`, the SDK defaults to `{"step_1": {"output": expected_output}}` for single-step workflows.

`test()` returns a list of result dicts with at least `name`, `passed`, and `output`. Recovery is disabled in test mode, so Postgres is not required.

## Example

Save as `stub_test_demo.py`:

```python
from arcflow import Agent, Workflow

researcher = Agent(
 name="researcher",
 role="Research",
 instructions="Research the topic.",
)

writer = Agent(
 name="writer",
 role="Writer",
 instructions="Write a summary.",
)

workflow = Workflow("stub-test-demo")
workflow.step(researcher).step(writer)

results = workflow.test(
 [
 {
 "name": "two-step happy path",
 "input": "Solar panel efficiency",
 "expected_output": "Final summary text",
 "stub_responses": {
 "step_1": {"output": "Fact one. Fact two."},
 "step_2": {"output": "Final summary text"},
 },
 }
 ]
)

for row in results:
 print(row["name"], "passed=", row["passed"], "output=", row["output"])
```

Run:

```bash
python stub_test_demo.py
```

You pinned both step outputs. The final string must match `expected_output` for `passed` to be `True`.

## Verify

| Check | Expected |
|-------|----------|
| Happy path case | `passed` is `True` |
| Wrong expected output | `passed` is `False` when final text differs |
| Failure then recovery stub | `fail_times` plus `then_output` succeeds without live retry config |

Failure recovery stub (single step):

```python
from arcflow import Agent, Workflow

agent = Agent(name="writer", role="Writer", instructions="Write.")
wf = Workflow("fail-times-demo").step(agent)

results = wf.test(
 [
 {
 "name": "recover after stub failures",
 "input": "hello",
 "expected_output": "recovered",
 "stub_responses": {
 "step_1": {"fail_times": 2, "then_output": "recovered"},
 },
 }
 ]
)
print(results[0]["passed"], results[0]["output"])
```

Expected: `True recovered`.

Optional pytest marker (requires `arcflow.testing.pytest_plugin`):

```python
import pytest

pytestmark = pytest.mark.arcflow_stub_responses(step_1={"output": "from-marker"})
```

See `sdk-python/tests/test_workflow_test.py` for full fixture examples.

## Next

[05 Retry and timeouts basics](05-retry-and-timeouts-basics.md) configures runtime retry and timeout limits that pair with test mode for failure drills.
