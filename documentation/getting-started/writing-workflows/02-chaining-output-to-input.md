# 02 Chaining output to input


## Before you start

Complete [01 Linear pipelines](01-linear-pipelines.md). Read [04 Context and prior steps](../writing-agents/04-context-and-prior-steps.md) if you have not already. That lesson introduces `ContextPolicy`; here we focus on why handoffs matter in multi-step workflows.

## Concept

In a linear pipeline, each step produces text output. When the runtime builds the prompt for a later step, it can include:

| Source | What it is |
|--------|------------|
| Run input | The original string passed to `workflow.run(input)` |
| Prior step output | Text from one or more earlier steps |

This is how step two "sees" step one's work without you manually copying strings in Python. The runtime assembles context before the agent runs.

Control that assembly with `ContextPolicy` on each `Agent`:

| Field | Purpose |
|-------|---------|
| `include_prior_steps` | `"last"` (only the previous step), `"all"` (all prior steps), or `"none"` |
| `include_run_input` | Whether the original run input is still visible |
| `max_prior_step_chars` | Cap on prior text length (minimum 256) |

Common pattern for research then write:

- Researcher: `include_prior_steps="none"`, `include_run_input=True` (focus on the user topic).
- Writer: `include_prior_steps="last"`, `include_run_input=False` (summarize research only, not the raw topic).

If you omit `context`, the runtime applies its default policy. Explicit policies make handoffs predictable before you switch from stub to a paid provider.

You do not pass prior output manually in `run()`. The chain is declarative: register steps in order and set policies per agent.

## Example

Save as `chain_handoff.py`:

```python
from arcflow import Agent, ContextPolicy, Workflow

researcher = Agent(
    name="researcher",
    role="Research",
    instructions="Research the topic and list key points.",
    context=ContextPolicy(
        include_prior_steps="none",
        include_run_input=True,
    ),
)

writer = Agent(
    name="writer",
    role="Writer",
    instructions="Write a summary based only on the research output.",
    context=ContextPolicy(
        include_prior_steps="last",
        include_run_input=False,
        max_prior_step_chars=8192,
    ),
)

workflow = Workflow("chain-handoff")
workflow.step(researcher).step(writer)

result = workflow.run("Microgrids for remote clinics")

print(result.output[:200], "..." if len(result.output) > 200 else "")
print(f"status={result.status} steps={result.step_count}")
```

Run:

```bash
python chain_handoff.py
```

The researcher consumes the run input. The writer is instructed to rely on prior step text; with `include_run_input=False`, the writer should not re-read the original topic verbatim in its prompt assembly.

## Verify

| Check | Expected |
|-------|----------|
| Script completes | No configuration error |
| `result.status` | `"completed"` |
| `result.step_count` | `2` |
| Invalid policy rejected | `ContextPolicy(include_prior_steps="sometimes")` raises `WorkflowConfigurationError` |

Sanity check for invalid policy:

```python
from arcflow import ContextPolicy
from arcflow.exceptions import WorkflowConfigurationError

try:
    ContextPolicy(include_prior_steps="sometimes")
except WorkflowConfigurationError:
    print("Invalid include_prior_steps rejected as expected")
```

## Next

[03 Graph workflows intro](03-graph-workflows-intro.md) covers conditional routing when linear order is not enough.
