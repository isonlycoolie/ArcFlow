# First linear workflow

**Audience:** `[developer]`

This walkthrough runs a multi-step linear pipeline with the Python SDK and stub provider. You define ordered agents, call `run()`, and confirm completion through status fields and trace events. No server, Postgres, or API keys are required.

## What this example demonstrates

A linear workflow chains agents in fixed order: each step receives the prior step output as context. The pattern matches Track A and production pipelines where routing is not conditional. The runnable script is [`examples/personal/blog_pipeline.py`](../../examples/personal/blog_pipeline.py): research, write, then SEO review on one topic string.

## Prerequisites

| Item | Value |
|------|-------|
| SDK | Python SDK built per [installation](../sdks/python/installation.md) |
| Provider | Default stub (no `OPENAI_API_KEY` required) |
| Infrastructure | None for embedded SDK |
| Tutorial track | [Track A](../tutorials/track-a-first-workflow.md) |

## Step 1: Inspect the script

Open [`examples/personal/blog_pipeline.py`](../../examples/personal/blog_pipeline.py). It registers three agents and three ordered steps:

```python
from arcflow import Agent, Workflow

researcher = Agent(name="researcher", role="researcher", instructions="Collect topic facts.")
writer = Agent(name="writer", role="writer", instructions="Draft a blog post.")
seo = Agent(name="seo", role="seo", instructions="Suggest title and meta description.")

wf = (
    Workflow("blog_pipeline")
    .step(researcher)
    .step(writer)
    .step(seo)
)
result = wf.run("Write about context assembly in agent workflows")
print(result.output)
```

For a two-step variant identical to Track A, see [Track A](../tutorials/track-a-first-workflow.md).

## Step 2: Run from repository root

```bash
python examples/personal/blog_pipeline.py
```

## Step 3: Verify run outcome

Append checks or run interactively:

```python
assert result.step_count == 3
assert result.status == "completed"
assert len(result.output) > 0
assert result.run_id
print("linear workflow checks passed")
```

| Field | Expected |
|-------|----------|
| `step_count` | `3` |
| `status` | `completed` (SDK lowercase) |
| `run_id` | UUID string |
| `output` | Non-empty string |

## Step 4: Verify trace events

```python
kinds = {e.get("event_kind") for e in result.trace_events}
required = {"WorkflowStarted", "StepCompleted", "WorkflowCompleted"}
missing = required - kinds
if missing:
    raise SystemExit(f"missing trace kinds: {missing}")
print("trace kinds ok:", sorted(kinds))
```

Optional structured trace:

```python
trace = wf.trace()
assert trace.run_id == result.run_id
assert len(trace) == 3
print(trace.summary())
```

## Expected output

Terminal output includes stub-generated blog text (content varies by version). You should also see three completed steps when printing `result.step_count` and a UUID when printing `result.run_id`. Exact prose is not part of pass criteria; structure is.

Example shape (values will differ):

```
[stub blog draft and SEO suggestions...]
