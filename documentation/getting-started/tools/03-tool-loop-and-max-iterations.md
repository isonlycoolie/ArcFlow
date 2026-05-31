# 03 Tool loop and max iterations


## Before you start

Complete [02 Attach tools to agents](02-attach-tools-to-agents.md). You should have at least one tool attached to an agent in a workflow step.

## Concept

When an agent has tools, the runtime may enter a **tool loop**: the model proposes a tool call, your `execute` function runs, and the result returns to the model until the step finishes or a limit is hit.

Bound that loop with `ToolExecutionConfig` on the agent:

```python
Agent(..., tool_execution=ToolExecutionConfig(mode="llm_select", max_iterations=5))
```

| Field | Purpose |
|-------|---------|
| `mode` | `"llm_select"` (default) lets the model choose tools; `"legacy_eager"` uses eager execution semantics |
| `max_iterations` | Maximum tool loop rounds for this agent (1 to 20 inclusive) |

Each iteration is one select-and-execute cycle. If the agent needs many tool calls, raise `max_iterations` within the allowed cap. If you want tighter cost control, lower it.

Invalid values raise `WorkflowConfigurationError` at construction time (for example `max_iterations=0` or an unknown `mode`).

Tool invocation timeouts are separate: each `Tool` has its own `timeout_seconds`. A slow `execute` function can raise `ToolExecutionError` even when loop iterations remain.

## Example

Save as `tool_loop_config.py`:

```python
from arcflow import Agent, Tool, ToolExecutionConfig, Workflow


def lookup_execute(payload: dict) -> str:
    topic = str(payload.get("topic", ""))
    return f"stub lookup result for {topic}"


lookup = Tool(
    name="lookup",
    description="Look up facts about a topic.",
    input_schema={
        "type": "object",
        "properties": {"topic": {"type": "string"}},
        "required": ["topic"],
    },
    execute=lookup_execute,
)

agent = Agent(
    name="researcher",
    role="Researcher",
    instructions="Research using lookup when needed.",
    tools=(lookup,),
    tool_execution=ToolExecutionConfig(
        mode="llm_select",
        max_iterations=3,
    ),
)

workflow = Workflow("tool-loop-demo")
workflow.step(agent)

result = workflow.run("Quantum dot displays")
print(result.output)
print(f"status={result.status}")
```

Run:

```bash
python tool_loop_config.py
```

## Verify

| Check | Expected |
|-------|----------|
| Valid config | Agent constructs without error |
| `max_iterations=0` | Raises `WorkflowConfigurationError` |
| `max_iterations=21` | Raises `WorkflowConfigurationError` |
| Invalid mode | `mode="auto"` raises `WorkflowConfigurationError` |

Bounds check:

```python
from arcflow import ToolExecutionConfig
from arcflow.exceptions import WorkflowConfigurationError

for bad in (0, 21):
    try:
        ToolExecutionConfig(max_iterations=bad)
    except WorkflowConfigurationError as err:
        print(f"max_iterations={bad} rejected:", err)
```

## Next

[04 Common tools bundle](04-common-tools-bundle.md) introduces prebuilt web and document tools you can attach instead of writing your own.
