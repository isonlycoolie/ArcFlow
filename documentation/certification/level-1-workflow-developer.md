# Level 1: Certified ArcFlow Workflow Developer

**Audience:** `[developer]`

**Title:** Certified ArcFlow Workflow Developer

**Prerequisite:** None. Start here if you write agent workflows in Python or TypeScript.

## What certified means at this level

You can build and run linear multi-step workflows with real or stub providers, configure agents with instructions and tools, read execution traces for basic lifecycle events, use `arcflow init` and `arcflow run` from the CLI, and diagnose `WorkflowConfigurationError` and `WorkflowExecutionError` without external help.

## Competencies covered

| Competency | Demonstration |
|------------|---------------|
| Linear workflows | Three or more ordered steps with handoff |
| Agent configuration | Instructions, role, optional tools |
| Providers | Stub for tests; switch to one real LLM provider |
| Traces | Identify `WorkflowStarted`, `StepCompleted`, `WorkflowCompleted` |
| CLI | `init` project and `run` a script locally |
| Errors | Fix config errors vs execution failures |

## Required reading

| Topic | Document |
|-------|----------|
| Product mental model | [What is ArcFlow](../concepts/what-is-arcflow.md) |
| Architecture | [Architecture overview](../concepts/architecture-overview.md) |
| Execution | [Execution model](../concepts/execution-model.md) |
| Install | [Install and build](../getting-started/install-and-build.md) |
| First workflow | [First workflow in five minutes](../getting-started/first-workflow-in-five-minutes.md) |
| Quickstart | [Python](../getting-started/quickstart-python.md) or [TypeScript](../getting-started/quickstart-typescript.md) |
| Linear workflows | [Linear workflows](../guides/workflows/linear-workflows.md) |
| Agents | [Defining agents](../guides/agents-and-tools/defining-agents.md) |
| Providers | [Provider configuration](../guides/agents-and-tools/provider-configuration.md) |
| Traces | [Execution traces](../guides/observability/execution-traces.md) |
| Tutorial | [Track A](../tutorials/track-a-first-workflow.md) |

## Tutorial track

Complete [Track A](../tutorials/track-a-first-workflow.md) with all verification assertions passing in your chosen language.

## Practical project

Build a three-step **research, analyze, summarize** pipeline:

| Step | Agent role | Output expectation |
|------|------------|-------------------|
| 1 Research | Gather facts on a topic you choose | Structured notes |
| 2 Analyze | Identify themes and risks | Analysis text |
| 3 Summarize | Executive summary | Final string returned from `run()` |

### Requirements

| Requirement | Detail |
|-------------|--------|
| Real LLM provider | At least one step uses a non-stub provider via env configuration |
| Tool | At least one agent registers a simple tool (calculator, datetime, or domain stub) |
| Trace verification | Script asserts required lifecycle event kinds after `run()` |
| CLI | Project creatable via `arcflow init`; workflow runnable via CLI |
| README | Lists env vars and verify command |

### Suggested starting points

| Resource | Link |
|----------|------|
| Blog pipeline sample | [`examples/personal/blog_pipeline.py`](../../examples/personal/blog_pipeline.py) |
| First linear doc | [first-linear-workflow](../examples/first-linear-workflow.md) |

Extend the blog pipeline pattern with an analyze step and real provider on one agent.

### Verification commands

```bash
# SDK run
python your_project/main.py

# Expected: exit 0, non-empty output, step_count >= 3

# CLI trace (replace RUN_ID)
cargo run -p arcflow-cli -- trace RUN_ID --format json
```

Pass criteria checklist:

| Check | Pass |
|-------|------|
| `status == completed` | yes |
| `step_count >= 3` | yes |
| Tool invoked at least once | visible in trace or logs |
| Real provider step | non-stub model id in config |
| Trace kinds include lifecycle trio | yes |
| Configuration error recovery | you can explain a fixed config mistake |

