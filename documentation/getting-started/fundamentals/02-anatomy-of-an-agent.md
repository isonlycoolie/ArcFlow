# Anatomy of an agent


## Before you start

Read [How ArcFlow thinks](01-how-arcflow-thinks.md) so the declare-vs-execute split is clear. Have the Python SDK installed per [Install and build](../install-and-build.md).

## Concept

An `Agent` is a behavioral unit you hand to a workflow. It is not a running process, a chat session, or a direct LLM client. It is a bundle of metadata the runtime uses when that step executes.

Three fields define the agent for most beginner workflows:

| Field | Purpose |
|-------|---------|
| `name` | Stable identifier in traces, logs, and error messages. Use short snake_case or single words (`researcher`, `writer`). |
| `role` | A coarse job label traces and events use (`research`, `write`, `summarize`). It does not select a model. |
| `instructions` | The system-facing prompt text for that agent. This is where you describe tone, format, and constraints. |

The constructor looks like this:

```python
Agent(name="...", role="...", instructions="...")
```

ArcFlow assigns each agent an internal UUID (`agent_id`) automatically. You reference agents by the object you created, not by typing that UUID yourself.

Optional fields exist for later (`tools`, `memory`, `context`, `model`), but the fundamentals track only needs the three strings above.

Validation runs at construction time, not at `run()`. If a required string is missing or only whitespace, Python raises `WorkflowConfigurationError` before any workflow executes. Messages follow a consistent pattern:

```
[ArcFlow] Agent <field> must be a non-empty string. Provide a meaningful <field> for this agent.
```

That format is deliberate: the first sentence states the rule; the second tells you how to fix it. Configuration errors always mean "fix the definition, then try again," not "retry the run."

## Minimal example

Save as `agent_anatomy.py`:

```python
from arcflow import Agent, Workflow

greeter = Agent(
 name="greeter",
 role="greet",
 instructions=(
 "Welcome the user by topic. "
 "Reply in two short sentences. "
 "Do not use bullet lists."
 ),
)

wf = Workflow("agent_demo")
wf.step(greeter)

result = wf.run("ArcFlow fundamentals")
print(result.output)
print(greeter) # repr shows name and role, not instructions
```

Run:

```bash
python agent_anatomy.py
```

## Verify

**Valid agent.** The script above should complete with `status=completed` (check via `print(result.status)` if you add it).

**Invalid name.** In a separate scratch file, try:

```python
from arcflow import Agent
from arcflow.exceptions import WorkflowConfigurationError

try:
 Agent(name=" ", role="test", instructions="ok")
except WorkflowConfigurationError as err:
 print(err)
```

You should see `[ArcFlow] Agent name must be a non-empty string` in the output. The same rule applies to `role` and `instructions`. Fix the empty field and construction succeeds without touching a workflow.

**Duplicate tools (preview).** If you later attach tools, each `tool.name` on one agent must be unique. Duplicate names raise `WorkflowConfigurationError` with `Duplicate tool name`. That is out of scope for this lesson but worth knowing when you extend agents.

## Next lesson

[Anatomy of a workflow](03-anatomy-of-a-workflow.md): chaining agents with `Workflow()`, `step()`, and `run()`, and reading `WorkflowResult`.
