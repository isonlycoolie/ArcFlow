# Tools


This track teaches how to declare tools, attach them to agents, bound the tool execution loop, and use the bundled common tools package. Examples use the stub provider unless noted.

## What you will learn

| Lesson | Topic |
|--------|-------|
| [01 Define a tool](01-define-a-tool.md) | `Tool(name, description, input_schema, execute)` |
| [02 Attach tools to agents](02-attach-tools-to-agents.md) | Pass tools on `Agent(..., tools=...)` |
| [03 Tool loop and max iterations](03-tool-loop-and-max-iterations.md) | `ToolExecutionConfig` and loop bounds |
| [04 Common tools bundle](04-common-tools-bundle.md) | `CommonTools.bundle()` for web and document helpers |

## Prerequisites

Complete [Install and build](../install-and-build.md), the [fundamentals](../fundamentals/) track, and at least [01 Linear pipelines](../writing-workflows/01-linear-pipelines.md). You should be comfortable defining agents and running `workflow.run(input)` with stub output.

Quick sanity check:

```bash
python -c "from arcflow import Agent, Tool, Workflow; print('ready')"
```

## How these lessons are structured

Every page follows the same sections: **Before you start**, **Concept**, **Example**, **Verify**, and **Next**. Run each example as a standalone script.

## After this track

| Goal | Next document |
|------|---------------|
| Full tool loop semantics | [Tool execution loop](../../guides/agents-and-tools/tool-execution-loop.md) |
| LangChain tool conversion | `arcflow.langchain.from_langchain_tool` |
| Agent fields reference | [Defining agents](../../guides/agents-and-tools/defining-agents.md) |
