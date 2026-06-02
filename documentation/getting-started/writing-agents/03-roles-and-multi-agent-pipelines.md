# 03 Roles and multi-agent pipelines


## Before you start

Complete [02 Instructions that work](02-instructions-that-work.md). You should be comfortable defining a single agent and calling `workflow.run(input)`. Re-read the step ordering section in [03 Anatomy of a workflow](../fundamentals/03-anatomy-of-a-workflow.md) if needed.

## Concept

### What `role` does

`role` is a short human-readable label (for example `"Research analyst"` or `"Editor"`). The runtime uses it alongside `name` when framing the agent in traces and prompt assembly. It is not a permission system and not a separate model. Think of it as the job title printed on the agent's name tag.

`name` should stay stable and machine-friendly (`researcher`, `writer`). `role` can read naturally in logs (`Research`, `Writer`).

### Multi-agent pipelines

A pipeline is a linear workflow with two or more steps. Each step runs one agent in registration order. Step one receives the run input you pass to `run()`. Step two runs after step one finishes. By default, later agents can see output from earlier steps (lesson 04 configures exactly what they see).

The canonical pattern is **research then write**: the first agent gathers or expands on the topic; the second agent produces a polished summary. This matches [First workflow in five minutes](../first-workflow-in-five-minutes.md) and [Track A](../../tutorials/track-a-first-workflow.md).

Each agent keeps its own `instructions`. The writer should not repeat the researcher's full job description. Tell the writer to summarize prior work, not to redo research from scratch.

## Example

Save as `two_agent_pipeline.py`:

```python
from arcflow import Agent, Workflow

researcher = Agent(
 name="researcher",
 role="Research",
 instructions="Research the given topic thoroughly. List key facts.",
)

writer = Agent(
 name="writer",
 role="Writer",
 instructions="Write a clear summary of the research. Use short paragraphs.",
)

workflow = Workflow("research-then-write")
workflow.step(researcher)
workflow.step(writer)

result = workflow.run("Analyze renewable energy trends")

print(result.output[:200], "..." if len(result.output) > 200 else "")
print(f"run_id={result.run_id}")
print(f"status={result.status} steps={result.step_count}")
```

Run:

```bash
python two_agent_pipeline.py
```

Registration order matters. `workflow.step(researcher)` before `workflow.step(writer)` ensures research runs first. The final string on `result.output` comes from the last step (the writer).

## Verify

| Check | Expected |
|-------|----------|
| `result.status` | `"completed"` |
| `result.step_count` | `2` |
| `result.run_id` | Non-empty UUID string |
| `result.output` | Non-empty text |

Optional: inspect trace metadata after the run (same pattern as Track A):

```python
for event in result.trace_events:
 print(event.get("kind"), event.get("step_id", ""))
```

You should see lifecycle events such as `WorkflowStarted`, `StepCompleted`, and `WorkflowCompleted`. Prompt text never appears in traces (trace data policy).

## Next

[04 Context and prior steps](04-context-and-prior-steps.md) introduces `ContextPolicy` so you control how much of the run input and prior step output each agent receives.
