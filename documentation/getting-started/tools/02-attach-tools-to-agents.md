# 02 Attach tools to agents


## Before you start

Complete [01 Define a tool](01-define-a-tool.md). You should know how to construct a `Tool` with `name`, `description`, `input_schema`, and `execute`.

## Concept

Tools attach to agents, not directly to workflows. Pass a tuple of `Tool` instances to the agent constructor:

```python
Agent(..., tools=(tool_a, tool_b))
```

The workflow still registers **agents** as steps (or graph nodes). When that step runs, the agent carries its tool list into the runtime.

Rules enforced at construction time:

| Rule | Error if violated |
|------|-------------------|
| Each entry must be a `Tool` instance | Type validation in binding layer |
| Tool names must be unique within one agent | `WorkflowConfigurationError` on duplicate `name` |
| At least zero tools is valid | Agents without tools behave like plain prompt steps |

Multiple agents in one workflow can each carry different tool sets. A researcher might have no tools while a worker agent exposes several.

The stub provider may not exercise every tool call path the way a live model does. Tool attachment validation still runs, and integration tests confirm the wiring end to end.

## Example

Two tools on one agent in a single-step workflow:

Save as `attach_tools.py`:

```python
from arcflow import Agent, Tool, Workflow


def add_execute(payload: dict) -> str:
 a = float(payload.get("a", 0))
 b = float(payload.get("b", 0))
 return str(a + b)


def greet_execute(payload: dict) -> str:
 name = str(payload.get("name", "world"))
 return f"Hello, {name}!"


add_numbers = Tool(
 name="add_numbers",
 description="Add two numbers.",
 input_schema={
 "type": "object",
 "properties": {
 "a": {"type": "number"},
 "b": {"type": "number"},
 },
 "required": ["a", "b"],
 },
 execute=add_execute,
)

greet = Tool(
 name="greet",
 description="Greet someone by name.",
 input_schema={
 "type": "object",
 "properties": {"name": {"type": "string"}},
 "required": ["name"],
 },
 execute=greet_execute,
)

agent = Agent(
 name="assistant",
 role="Assistant",
 instructions="Help the user using available tools.",
 tools=(add_numbers, greet),
)

workflow = Workflow("attach-tools-demo")
workflow.step(agent)

result = workflow.run("Demonstrate tool wiring")
print(result.output)
print(f"status={result.status}")
```

Run:

```bash
python attach_tools.py
```

## Verify

| Check | Expected |
|-------|----------|
| Script completes | No configuration error |
| Duplicate tool names | Second tool with same `name` on one agent raises `WorkflowConfigurationError` |

Duplicate name check:

```python
from arcflow import Agent, Tool
from arcflow.exceptions import WorkflowConfigurationError

def stub_execute(_: dict) -> str:
 return "ok"

t1 = Tool("dup", "first", {"type": "object"}, stub_execute)
t2 = Tool("dup", "second", {"type": "object"}, stub_execute)

try:
 Agent(name="a", role="A", instructions="Run.", tools=(t1, t2))
except WorkflowConfigurationError as err:
 print(err)
```

## Next

[03 Tool loop and max iterations](03-tool-loop-and-max-iterations.md) bounds how many tool rounds an agent may take in one step.
