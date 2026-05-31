# Writing workflows


This track teaches how to build ArcFlow workflows beyond a single agent step: linear pipelines, controlled handoffs between steps, graph routing, deterministic testing, and basic retry and timeout configuration. Each lesson is runnable with the built-in stub provider unless noted otherwise.

## What you will learn

| Lesson | Topic |
|--------|-------|
| [01 Linear pipelines](01-linear-pipelines.md) | Register ordered steps and read `WorkflowResult` |
| [02 Chaining output to input](02-chaining-output-to-input.md) | How prior step text reaches the next agent |
| [03 Graph workflows intro](03-graph-workflows-intro.md) | Nodes, edges, conditions, and join basics |
| [04 Testing with stub responses](04-testing-with-stub-responses.md) | `workflow.test()` and `stub_responses` |
| [05 Retry and timeouts basics](05-retry-and-timeouts-basics.md) | `retry()`, `timeout()`, and `step_timeout()` |

## Prerequisites

Complete [Install and build](../install-and-build.md) and the [fundamentals](../fundamentals/) track. You should also finish [Writing agents](../writing-agents/) so `Agent`, `ContextPolicy`, and multi-step pipelines are familiar.

Quick sanity check:

```bash
python -c "from arcflow import Agent, Workflow; print('ready')"
```

## How these lessons are structured

Every page follows the same sections: **Before you start**, **Concept**, **Example**, **Verify**, and **Next**. Run each example as a standalone script. No API keys are required until you connect a live provider in [Python quickstart](../quickstart-python.md).

## After this track

| Goal | Next document |
|------|---------------|
| Guided trace verification | [Track A: First workflow](../../tutorials/track-a-first-workflow.md) |
| Full graph patterns | [Track D: Graph workflows](../../tutorials/track-d-graph-workflows.md) |
| Recovery and resume | [Recovery and resume](../../guides/reliability/recovery-and-resume.md) |
| Tool-calling agents | [Tools track](../tools/) |
