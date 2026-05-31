# 05 Retry and timeouts basics

**Audience:** `[developer]`

## Before you start

Complete [04 Testing with stub responses](04-testing-with-stub-responses.md). You should know how to call `workflow.test()` with `stub_responses`. Read [When something fails](../fundamentals/05-when-something-fails.md) for the difference between configuration and execution errors.

## Concept

ArcFlow applies retry and timeout policy in the **Rust runtime**, not in your Python business logic. Configure limits on the workflow **before** `run()` or `test()`:

| Method | Purpose |
|--------|---------|
| `workflow.retry(max_attempts, backoff=...)` | Retry transient step failures up to `max_attempts` (max 50) |
| `workflow.timeout(seconds)` | Fail the whole run if total elapsed time exceeds the limit |
| `workflow.step_timeout(seconds)` | Fail a single step if it exceeds the limit |

Backoff strategies live in `arcflow.retry`: `ExponentialBackoff` (default), `ConstantBackoff`, and `LinearBackoff`. Retry logic runs inside the engine; the Python SDK only serializes the config.

When retries are exhausted, the runtime raises `RetryExhaustedError` with `[ArcFlow]` context and `attempts_made`. When a timeout fires, expect `WorkflowTimeoutError` with a `timeout_type` of `"workflow"` or `"step"`.

For CI without a live provider, combine `retry()` with `workflow.test()` and `stub_responses` entries that use `fail_times` and `then_output`. That proves retry wiring without HTTP mocks.

Live provider retries (rate limits, 5xx responses) need a real provider such as `OpenAI` and are covered in [Retry and backoff](../../guides/reliability/retry-and-backoff.md).

## Example

Save as `retry_basics.py`:

```python
from arcflow import Agent, Workflow
from arcflow.retry import ConstantBackoff

agent = Agent(
    name="worker",
    role="Worker",
    instructions="Process the input.",
)

workflow = (
    Workflow("retry-basics")
    .retry(3, backoff=ConstantBackoff(delay_ms=1))
    .step(agent)
)

results = workflow.test(
    [
        {
            "name": "succeeds on third attempt",
            "input": "process me",
            "expected_output": "done",
            "stub_responses": {
                "step_1": {"fail_times": 2, "then_output": "done"},
            },
            "assert_retries": 3,
        }
    ]
)

row = results[0]
print("passed=", row["passed"], "output=", row["output"])
print("attempts_made=", row.get("attempts_made"))
```

Run:

```bash
python retry_basics.py
```

The stub fails twice, succeeds on the third attempt, and `assert_retries` checks that the engine made exactly three attempts.

Timeout configuration (set before run; enforcement requires a slow step or live call):

```python
from arcflow import Agent, Workflow

agent = Agent(name="slow", role="Slow", instructions="Work carefully.")
wf = (
    Workflow("timeout-demo")
    .step_timeout(30.0)
    .timeout(120.0)
    .step(agent)
)
```

## Verify

| Check | Expected |
|-------|----------|
| Retry test case | `passed` is `True`, `attempts_made` is `3` |
| `retry(0)` | Raises `WorkflowConfigurationError` |
| Retry after `run()` | Calling `retry()` after the first run raises configuration error |

Invalid retry:

```python
from arcflow import Workflow
from arcflow.exceptions import WorkflowConfigurationError

try:
    Workflow("bad").retry(0)
except WorkflowConfigurationError as err:
    print(err)
```

## Next

| Goal | Document |
|------|----------|
| Tool-calling agents | [Tools track](../tools/) |
| Full retry and backoff reference | [Retry and backoff](../../guides/reliability/retry-and-backoff.md) |
| Live provider fault tolerance tests | `sdk-python/tests/integration/test_fault_tolerance.py` |

## Source

`sdk-python/arcflow/workflow.py` (`retry`, `timeout`, `step_timeout`); `sdk-python/arcflow/retry.py`; `sdk-python/tests/test_workflow_test_retry.py`; `sdk-python/tests/integration/test_fault_tolerance.py`; [Retry and backoff](../../guides/reliability/retry-and-backoff.md).
