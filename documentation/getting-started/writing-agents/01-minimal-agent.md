# 01 Minimal agent


## Before you start

You should have the Python SDK installed ([Install and build](../install-and-build.md#python-sdk-maturin--pip)) and have read [03 Anatomy of a workflow](../fundamentals/03-anatomy-of-a-workflow.md). That page explains how a `Workflow` holds steps and how `run(input)` produces `result.output`. Here we focus on the object each step references: the `Agent`.

## Concept

An agent is a definition, not a running process. You describe who the agent is and what it should do; the ArcFlow runtime executes that definition when the workflow reaches the step that references it.

The Python constructor requires three strings:

| Field | Purpose |
|-------|---------|
| `name` | Stable identifier in traces and logs (use lowercase, no spaces) |
| `role` | Short label that frames the agent's job in the prompt |
| `instructions` | The task prompt: what to produce and any constraints |

Empty strings are rejected at construction time with `WorkflowConfigurationError`. The runtime assigns a UUID internally; you do not need to pass an `id` for local scripts.

With no provider configured, the stub runtime returns deterministic placeholder text. That is enough to confirm your agent wiring before you add API keys.

## Example

Save as `minimal_agent.py`:

```python
from arcflow import Agent, Workflow

summarizer = Agent(
 name="summarizer",
 role="Summarizer",
 instructions="Summarize the user topic in two short paragraphs.",
)

workflow = Workflow("minimal-agent-demo")
workflow.step(summarizer)

result = workflow.run("Benefits of local-first software")
print(result.output)
print(f"status={result.status} steps={result.step_count}")
```

Run:

```bash
python minimal_agent.py
```

You defined one agent, registered it as the only step, and passed a run input string. The runtime executed that single step and returned its output on `result.output`.

## Verify

| Check | Expected |
|-------|----------|
| Script exits without exception | Yes |
| First `print` line | Non-empty text |
| `result.status` | `"completed"` |
| `result.step_count` | `1` |

If construction fails with `WorkflowConfigurationError`, one of the three required strings is blank or whitespace only.

## Next

[02 Instructions that work](02-instructions-that-work.md) covers how to write `instructions` that produce useful behavior when you switch from stub to a real provider.
