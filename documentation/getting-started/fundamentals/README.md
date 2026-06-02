# Fundamentals track


## What this track is

The fundamentals track is a short, ordered path through the ideas ArcFlow assumes you already know before you read longer guides or certification material. Each lesson is one concept, one minimal Python example, and one verification step. You declare agents and steps in Python; the Rust runtime in `arcflow-core` executes them. Nothing in this track requires an LLM API key.

The track takes about **30 minutes** if you install the SDK once and run each example as you read. If you have not built the Python package yet, start with [Install and build](../install-and-build.md) and return here when `from arcflow import Agent, Workflow` succeeds.

## How to use it

Read the lessons in order. Each file builds on the previous one. Copy the minimal example into a local script, run it, and complete the Verify section before moving on. Examples run locally with no API keys; output is deterministic placeholder text until you add a live provider in lesson 4.

| Order | Lesson | What you learn |
|-------|--------|----------------|
| 1 | [How ArcFlow thinks](01-how-arcflow-thinks.md) | Declaration vs execution, workflow specification, default runtime |
| 2 | [Anatomy of an agent](02-anatomy-of-an-agent.md) | `name`, `role`, `instructions`, validation |
| 3 | [Anatomy of a workflow](03-anatomy-of-a-workflow.md) | `Workflow()`, `step()`, `run()`, `WorkflowResult` |
| 4 | [Default runtime vs live LLM](04-stub-vs-live-provider.md) | No API key path, switching to OpenAI |
| 5 | [When something fails](05-when-something-fails.md) | Configuration vs execution errors, reading messages |

## Before you start

You need Python 3.9 or newer, the Rust toolchain, and a working install of the `arcflow` package from this repository. The [Install and build](../install-and-build.md) page walks through `maturin develop` in `sdk-python/` on macOS, Linux, and Windows.

Quick sanity check:

```bash
python -c "from arcflow import Agent, Workflow; print('ready')"
```

If that prints `ready`, open [lesson 1](01-how-arcflow-thinks.md).

## After this track

| Goal | Document |
|------|----------|
| Fastest end-to-end run | [First workflow in five minutes](../first-workflow-in-five-minutes.md) |
| OpenAI and trace reading | [Python quickstart](../quickstart-python.md) |
| Guided verification with trace events | [Track A: First workflow](../../tutorials/track-a-first-workflow.md) |
