# Writing agents


This track teaches how to define agents in Python and chain them into multi-step workflows. Each lesson is short, runnable with the built-in stub provider, and builds on the [Fundamentals](../fundamentals/) series.

## What you will learn

| Lesson | Topic |
|--------|-------|
| [01 Minimal agent](01-minimal-agent.md) | The three required fields: `name`, `role`, `instructions` |
| [02 Instructions that work](02-instructions-that-work.md) | Writing prompts the runtime can act on |
| [03 Roles and multi-agent pipelines](03-roles-and-multi-agent-pipelines.md) | How `role` frames behavior and how steps hand off output |
| [04 Context and prior steps](04-context-and-prior-steps.md) | `ContextPolicy` basics: what each agent sees from earlier steps |

## Prerequisites

Complete [Install and build](../install-and-build.md) so `from arcflow import Agent, Workflow` works in your virtual environment. Read [03 Anatomy of a workflow](../fundamentals/03-anatomy-of-a-workflow.md) first so terms like step, run input, and `result.output` are already familiar.

## How these lessons are structured

Every page follows the same sections: **Before you start**, **Concept**, **Example**, **Verify**, and **Next**. Run each example as a standalone script. No API keys are required until you move on to [Python quickstart](../quickstart-python.md).

## After this track

| Goal | Next document |
|------|---------------|
| Verify trace events and status checks | [Track A: First workflow](../../tutorials/track-a-first-workflow.md) |
| Full agent fields (tools, memory, provider) | [Defining agents](../../guides/agents-and-tools/defining-agents.md) |
| Deep dive on context assembly | [Context policies](../../guides/agents-and-tools/context-policies.md) |
