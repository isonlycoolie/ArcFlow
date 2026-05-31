# How ArcFlow thinks


## Before you start

Complete [Install and build](../install-and-build.md) so `from arcflow import Agent, Workflow` works in your virtual environment. You do not need API keys for this lesson.

## Concept

Most workflow libraries blur two jobs: describing what should happen, and actually running it. ArcFlow keeps them separate on purpose.

In Python you **declare** agents (who does the work) and register them as ordered steps on a `Workflow`. You call `run(input)` when you want execution to start. Python does not call an LLM, loop over steps, or manage retries. Those concerns live in the Rust runtime crate `arcflow-core`.

When `run()` fires, the Python SDK serializes your workflow into **RCS** (Runtime Contract Specification) JSON: agent definitions, step order, run input, and execution config. That payload crosses the native binding boundary into Rust. `WorkflowEngine` validates the shape, picks a provider, runs step one, then step two, and returns a result object back to Python.

For learning and CI, the default in-process agent backend returns deterministic placeholder text with no network calls. The same declaration path works when you pass `provider=OpenAI(model="gpt-4o")`; only the execution backend changes, not how you build the workflow.

The mental model in one line: **your script declares structure, RCS carries it, Rust executes it.**

```
Python script  →  RCS JSON  →  arcflow-core (Rust)  →  WorkflowResult back to Python
     │                │                    │
  Agent,          versioned           default or live
  Workflow,        contract            provider per step
  run()
```

This split is why Python, TypeScript, and the HTTP server can share behavior: they all speak RCS to the same engine. You can read more in [The RCS contract](../../concepts/the-rcs-contract.md) when you want the schema-level detail. For now, treat RCS as the wire format your declarations become at run time.

## Minimal example

Save as `think_arcflow.py`:

```python
from arcflow import Agent, Workflow

# Declaration only: no LLM call happens here.
analyst = Agent(
    name="analyst",
    role="analyze",
    instructions="Break the topic into three bullet points.",
)

pipeline = Workflow("fundamentals_demo")
pipeline.step(analyst)

# Execution starts here; Python sends RCS to arcflow-core.
result = pipeline.run("Solar panel efficiency trends")

print(result.output)
print(f"status={result.status} steps={result.step_count}")
```

Run:

```bash
python think_arcflow.py
```

You should see non-empty output on the first line and `status=completed steps=1` on the second. Exact wording can vary by runtime version; the integration tests only require `len(result.output) > 0`.

## Verify

Confirm the separation yourself:

1. Add a `print("about to run")` immediately before `pipeline.run(...)`.
2. Add another `print("run finished")` after it.
3. Run the script again.

Execution happens inside `run()`. If you comment out the `run()` line, the prints around it never show output because Rust never received a run request.

Optional: import succeeds without touching the network. That is expected. The default path needs no `OPENAI_API_KEY`.

## Next lesson

[Anatomy of an agent](02-anatomy-of-an-agent.md): what `name`, `role`, and `instructions` mean, and how `Agent()` validates them.
