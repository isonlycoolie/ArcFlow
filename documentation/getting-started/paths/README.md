# Getting started paths


## What this section is

Paths are end-to-end routes through ArcFlow for different roles and deployment shapes. Each path is a self-contained document you can finish in one sitting. Pick the path that matches how you plan to run workflows, then follow the linked integrating lessons when you need server, HITL, or external callback behavior.

## Choose a path

| Path | Audience | Time | Outcome |
|------|----------|------|---------|
| [First workflow in five minutes](../first-workflow-in-five-minutes.md) | Developer | ~5 min | Two-agent workflow in Python, no Docker |
| [Python quickstart](../quickstart-python.md) | Developer | ~20 min | Install, default run, optional OpenAI, traces |
| [TypeScript quickstart](../quickstart-typescript.md) | Developer | ~20 min | Node SDK, async patterns, parity with Python |
| [Server API quickstart](../quickstart-server-api.md) | Platform | ~30 min | Docker stack, curl create/poll/trace |
| [Static site chatbot](static-site-chatbot.md) | Frontend + operator | ~45 min | Full E2E: server, admin, ingest, publish, embed |

## Recommended order

If you are new to ArcFlow:

1. [First workflow in five minutes](../first-workflow-in-five-minutes.md) to confirm SDK wiring.
2. [Python quickstart](../quickstart-python.md) or [TypeScript quickstart](../quickstart-typescript.md) for traces and optional live providers.
3. [Integrating track](../integrating/README.md) when you need HTTP server, callbacks, or HITL.
4. [Static site chatbot](static-site-chatbot.md) when you ship a browser chat widget with Relay.

## Before any path

Install the SDK or confirm Docker depending on your path:

| Requirement | Document |
|-------------|----------|
| Python / Rust toolchain | [Install and build](../install-and-build.md) |
| Conceptual background | [Fundamentals](../fundamentals/README.md) |
| Agent authoring detail | [Writing agents](../writing-agents/README.md) |

## After these paths

| Goal | Document |
|------|----------|
| Guided trace verification | [Track A: First workflow](../../tutorials/track-a-first-workflow.md) |
| Server tutorial with checklist | [Track B: Server API](../../tutorials/track-b-server-api.md) |
| Static product tutorial | [Track F: Static product](../../tutorials/track-f-static-product.md) |
| Certification ladder | [Certification overview](../../certification/overview.md) |
