# Default runtime vs live LLM

**Audience:** `[developer]`

## Before you start

Complete [Anatomy of a workflow](03-anatomy-of-a-workflow.md) so `Workflow`, `step()`, and `run()` are familiar. Examples in this lesson need only the SDK from [Install and build](../install-and-build.md). The live OpenAI section needs a valid `OPENAI_API_KEY` when you choose to run it.

## Concept

A **provider** is the backend that produces agent output during a step. ArcFlow abstracts providers so the same workflow declaration can run in CI without network access and in production with a real model.

When you call `run(input)` and omit `provider`, the runtime uses the **default in-process agent backend**. It returns deterministic placeholder text derived from the agent's role and the run input. No API key is read and no HTTP request leaves your machine.

When you are ready for a live model, pass a provider object to `run()`:

```python
result = workflow.run("topic", provider=OpenAI(model="gpt-4o"))
```

The provider applies to the run (and can be overridden per advanced configs later). Your agents, steps, and workflow shape stay the same; only the execution backend changes from the default path to OpenAI.

OpenAI reads the API key from the environment variable `OPENAI_API_KEY`. The SDK never accepts raw secrets in source code. Set the variable in your shell or `.env` loader before running:

```bash
export OPENAI_API_KEY="sk-your-key-here"   # macOS / Linux
# $env:OPENAI_API_KEY="sk-your-key-here"   # PowerShell
```

Other built-in provider classes (`Anthropic`, `Gemini`) follow the same pattern with their own environment variables. This lesson focuses on the default path vs OpenAI because that is the most common first switch.

Provider misconfiguration (empty model name, invalid temperature) raises `ProviderConfigurationError` before the run starts. Provider failures during a live call raise `ProviderExecutionError` with context you can log.

## Minimal example (no API key)

Save as `default_runtime.py`:

```python
from arcflow import Agent, Workflow

agent = Agent(
    name="summarizer",
    role="summarize",
    instructions="Summarize in three sentences.",
)

wf = Workflow("default_demo")
wf.step(agent)

# provider omitted: default backend is used automatically
result = wf.run("Battery recycling methods")
print("output:", result.output)
print("completed:", result.status)
```

Run without any API keys:

```bash
python default_runtime.py
```

You should see non-empty output and `completed: completed`.

## Minimal example (live OpenAI, optional)

Save as `openai_live.py` and run only when `OPENAI_API_KEY` is set:

```python
import os

from arcflow import Agent, OpenAI, Workflow

if not os.environ.get("OPENAI_API_KEY"):
    raise SystemExit("Set OPENAI_API_KEY before running this script.")

agent = Agent(
    name="summarizer",
    role="summarize",
    instructions="Summarize in three sentences.",
)

wf = Workflow("openai_demo")
wf.step(agent)

result = wf.run(
    "Battery recycling methods",
    provider=OpenAI(model="gpt-4o"),
)
print("live output:", result.output)
print("completed:", result.status)
```

Run:

```bash
python openai_live.py
```

Live output varies by model and temperature. Default-runtime output for the same workflow shape will differ in wording but follows the same `WorkflowResult` fields.

## Verify

**Default path.** `default_runtime.py` completes with no environment variables beyond Python itself.

**Explicit equivalence.** These two calls behave the same for beginner linear workflows: `run("input")` and `run("input", provider=None)`.

**Live path.** With a valid key, `openai_live.py` returns model-generated text. With a missing or invalid key, expect `ProviderConfigurationError` or `ProviderExecutionError` depending on when validation fails. Read the `[ArcFlow]` message for the remediation hint.

**Same workflow, two modes.** Keep one `Workflow` definition and toggle only the `provider=` argument. That is the intended design: declaration stable, provider swappable.

## Next lesson

[When something fails](05-when-something-fails.md): `WorkflowConfigurationError` vs `WorkflowExecutionError`, and how to read ArcFlow error messages.

**Source:** `sdk-python/arcflow/provider.py`, `sdk-python/README.md`, [Python quickstart](../quickstart-python.md); capabilities reference §16, §16.2.
