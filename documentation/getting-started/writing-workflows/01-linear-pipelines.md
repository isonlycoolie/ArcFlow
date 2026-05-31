# 01 Linear pipelines


## Before you start

Complete [03 Anatomy of a workflow](../fundamentals/03-anatomy-of-a-workflow.md) and [03 Roles and multi-agent pipelines](../writing-agents/03-roles-and-multi-agent-pipelines.md). You should already have a two-step pipeline running with the stub provider.

## Concept

A **linear pipeline** is a workflow where each agent runs once, in registration order. Step one receives the string you pass to `run(input)`. Step two runs after step one finishes. Step three runs after step two, and so on.

The Python API stays compact:

1. Create `Workflow("name")`.
2. Call `workflow.step(agent)` for each agent in order. Each call returns the same workflow object, so you can chain: `workflow.step(a).step(b).step(c)`.
3. Call `workflow.run(input)` once.

The last step's text becomes `result.output`. Metadata such as `run_id`, `step_count`, and `status` lives on `WorkflowResult`.

Linear mode is the default. You do not pass `graph=True`. Graph workflows (lesson 03) use a different builder API.

Naming the workflow (`Workflow("research_pipeline")` instead of the default `"default"`) helps when you read traces or compare runs in logs.

## Example

Three agents in a research, draft, edit chain:

Save as `linear_pipeline.py`:

```python
from arcflow import Agent, Workflow

researcher = Agent(
    name="researcher",
    role="Research",
    instructions="List five facts about the topic.",
)

drafter = Agent(
    name="drafter",
    role="Draft",
    instructions="Turn the facts into a short article outline.",
)

editor = Agent(
    name="editor",
    role="Edit",
    instructions="Polish the outline into final prose.",
)

workflow = (
    Workflow("research-draft-edit")
    .step(researcher)
    .step(drafter)
    .step(editor)
)

result = workflow.run("Urban gardening in small spaces")

print(result.output[:200], "..." if len(result.output) > 200 else "")
print(f"run_id={result.run_id}")
print(f"status={result.status} steps={result.step_count}")
```

Run:

```bash
python linear_pipeline.py
```

Registration order is execution order. The editor runs last, so `result.output` reflects the editor step.

## Verify

| Check | Expected |
|-------|----------|
| Script exits without exception | Yes |
| `result.status` | `"completed"` |
| `result.step_count` | `3` |
| `result.output` | Non-empty text |
| `result.run_id` | UUID-shaped string |

Optional trace skim:

```python
for event in result.trace_events:
    kind = event.get("event_kind") or event.get("kind")
    if kind in ("StepStarted", "StepCompleted", "WorkflowCompleted"):
        print(kind, event.get("step_index", ""))
```

You should see three step lifecycles before `WorkflowCompleted`. Prompt text never appears in trace payloads (SEC-1).

## Next

[02 Chaining output to input](02-chaining-output-to-input.md) explains how each step receives text from earlier steps and how to control that with `ContextPolicy`.
