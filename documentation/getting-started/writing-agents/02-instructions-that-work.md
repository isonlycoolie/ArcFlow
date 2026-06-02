# 02 Instructions that work


## Before you start

Complete [01 Minimal agent](01-minimal-agent.md). You should understand the three required fields on `Agent(name, role, instructions)` and how a single step runs with the stub provider.

## Concept

`instructions` is the field the runtime treats as the agent's task prompt. `name` and `role` label the agent in traces; `instructions` tells the model what to do with the run input.

Good instructions share three properties:

1. **Outcome is explicit.** State the deliverable (a summary, a bullet list, a classification label) rather than a vague goal like "help the user."
2. **Constraints are bounded.** Length, format, and tone reduce drift. "Three bullet points, plain language, no jargon" beats "be concise."
3. **Scope matches one step.** One agent should do one job. If you find yourself writing "first research, then write, then proofread" in a single block, split that work across multiple agents in lesson 03.

The stub provider does not interpret nuance the way a live LLM will. Still, writing clear instructions now means less rework when you enable a real provider in [Python quickstart](../quickstart-python.md).

## Example

Compare a vague instruction with a bounded one. Both run on the stub; the pattern is what matters for production.

Save as `instructions_demo.py`:

```python
from arcflow import Agent, Workflow

vague = Agent(
 name="helper",
 role="Assistant",
 instructions="Help with the topic.",
)

specific = Agent(
 name="briefing_writer",
 role="Briefing writer",
 instructions=(
 "Write a briefing note on the topic. "
 "Use exactly three bullet points. "
 "Each bullet is one sentence. "
 "Do not include a title or preamble."
 ),
)

Workflow("vague").step(vague).run("Quarterly sales review")
Workflow("specific").step(specific).run("Quarterly sales review")

print("Both agents constructed and ran successfully.")
print("When you enable a real provider, the specific instructions will constrain output shape.")
```

Run:

```bash
python instructions_demo.py
```

The script proves two agents with different instruction quality both serialize and execute. After you add a provider, re-run with the specific agent and compare output structure against the vague one.

## Verify

| Check | Expected |
|-------|----------|
| Script completes | No `WorkflowConfigurationError` |
| Both workflows run | Stub output on each (content may differ by runtime version) |
| Instruction strings | Non-empty after `.strip()` |

Try removing all text from `instructions` and confirm construction raises `WorkflowConfigurationError` with a message naming the field.

## Next

[03 Roles and multi-agent pipelines](03-roles-and-multi-agent-pipelines.md) adds a second agent and shows how ordered steps form a pipeline.
