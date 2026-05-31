# 01 Define a tool

**Audience:** `[developer]`

## Before you start

Complete [02 Anatomy of an agent](../fundamentals/02-anatomy-of-an-agent.md) and [01 Linear pipelines](../writing-workflows/01-linear-pipelines.md). You should have a single-step workflow running with the stub provider.

## Concept

A **tool** is a named capability an agent can invoke during a step. You declare it in Python; the runtime validates inputs against a JSON Schema and calls your `execute` function when the agent selects the tool.

The constructor signature is:

```python
Tool(name, description, input_schema, execute, timeout_seconds=30.0)
```

| Argument | Purpose |
|----------|---------|
| `name` | Stable identifier (lowercase, no spaces); must be unique per agent |
| `description` | Human-readable summary shown to the model during tool selection |
| `input_schema` | JSON Schema object describing allowed arguments |
| `execute` | Callable accepting one `dict` and returning a `str` result |
| `timeout_seconds` | Per-invocation limit (must be positive) |

Validation happens at construction time:

- Empty `name` or `description` raises `ToolConfigurationError`.
- Non-dict or non-serializable `input_schema` raises `ToolConfigurationError`.
- Non-callable `execute` raises `ToolConfigurationError`.

The `execute` function receives parsed arguments as a dict (keys match schema properties). Return a string; the runtime feeds that back into the agent loop.

Tools are definitions, like agents. They do not run until a workflow step executes and the agent loop selects them.

## Example

An echo tool that returns a message field:

Save as `define_tool.py`:

```python
from arcflow import Agent, Tool, Workflow


def echo_execute(payload: dict) -> str:
    message = str(payload.get("message", ""))
    return f"echo:{message}"


echo = Tool(
    name="echo",
    description="Return the message field unchanged with an echo prefix.",
    input_schema={
        "type": "object",
        "properties": {
            "message": {"type": "string", "description": "Text to echo"},
        },
        "required": ["message"],
    },
    execute=echo_execute,
)

agent = Agent(
    name="worker",
    role="Worker",
    instructions="Use the echo tool when helpful.",
    tools=(echo,),
)

workflow = Workflow("define-tool-demo")
workflow.step(agent)

result = workflow.run("hello-tools")
print(result.output)
print(f"status={result.status} steps={result.step_count}")
```

Run:

```bash
python define_tool.py
```

Stub runs may not always invoke the tool the way a live model would, but the workflow still completes and proves tool registration.

## Verify

| Check | Expected |
|-------|----------|
| Valid tool construction | No exception |
| Invalid schema type | `Tool(..., input_schema="bad", ...)` raises `ToolConfigurationError` |
| Empty name | `Tool(name="", ...)` raises `ToolConfigurationError` |

