# Getting started curriculum

**Audience:** `[developer]` `[operator]` `[frontend]`

ArcFlow workflows are written as simple declarations: you define agents, chain them into a workflow, and call `run()`. The Rust runtime handles scheduling, tools, memory, traces, and recovery. This curriculum teaches that model step by step: one concept per lesson, each with a copy-paste script you run locally on your machine.

Start with [Install and build](install-and-build.md) if you have not built the Python or TypeScript SDK yet.

## Recommended order

| Step | Track | Time (approx.) | Outcome |
|------|-------|----------------|---------|
| 1 | [Fundamentals](fundamentals/README.md) | 30 min | Mental model: Agent, Workflow, `run()` |
| 2 | [Writing agents](writing-agents/README.md) | 25 min | Instructions, roles, context policy |
| 3 | [Writing workflows](writing-workflows/README.md) | 35 min | Linear pipelines, graph intro, testing |
| 4 | [Tools](tools/README.md) | 25 min | Define tools, attach to agents, tool loop |
| 5 | [Memory](memory/README.md) | 20 min | Session, shared, vector setup |
| 6 | [RAG](rag/README.md) | 30 min | Ingest, retrieve, wire into agents |
| 7 | [Integrating](integrating/README.md) | 25 min | Embedded SDK vs server, HITL, callbacks |
| 8 | [Outcome paths](paths/README.md) | varies | End-to-end goals (static site, server API) |

You do not need every track before shipping. After **Fundamentals** and **Writing workflows**, you can jump to an [outcome path](paths/README.md) that matches your job.

## Outcome paths (pick one goal)

| Goal | Path |
|------|------|
| Fastest first run | [First workflow in five minutes](first-workflow-in-five-minutes.md) |
| Python app with optional live LLM | [Python quickstart](quickstart-python.md) |
| TypeScript / Node service | [TypeScript quickstart](quickstart-typescript.md) |
| HTTP integration (curl, backend team) | [Server API quickstart](quickstart-server-api.md) |
| Public website chat widget | [Static site chatbot](paths/static-site-chatbot.md) |

## The ArcFlow pattern (every lesson uses this)

```python
from arcflow import Agent, Workflow

agent = Agent(
    name="worker",
    role="assistant",
    instructions="Do one clear task with the user input.",
)

workflow = Workflow("my_flow")
workflow.step(agent)

result = workflow.run("your input here")  # default: no API key required
print(result.output)
```

Layers you add later: more steps, `Tool(...)`, `MemoryConfig(...)`, `provider=OpenAI(...)`, graph mode, server `runtime=`, HITL, external callbacks.

## Where to go after this section

| Need | Document |
|------|----------|
| Reference depth | [Guides](../guides/workflows/linear-workflows.md) |
| API signatures | [Python SDK reference](../sdks/python/api-reference.md) |
| Guided tracks A–H | [Tutorials](../tutorials/track-a-first-workflow.md) |
| Architecture | [Concepts](../concepts/what-is-arcflow.md) |

## Source

Capabilities reference §16, §28; `sdk-python/arcflow/agent.py`, `sdk-python/arcflow/workflow.py`.
