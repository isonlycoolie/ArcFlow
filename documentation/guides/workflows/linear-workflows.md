**Audience:** `[developer]`

# Linear workflows

Linear execution is the default path for multi-agent pipelines where each step runs in a fixed order. You set `execution_mode: "linear"` on the workflow, assign each step an `order` integer, and the engine sorts and runs them sequentially. Output from earlier steps can flow into later agents through [context policies](../agents-and-tools/context-policies.md) and shared memory.

If you have not run a workflow yet, start with [First workflow in five minutes](../../getting-started/first-workflow-in-five-minutes.md). For the shared execution concepts (state machine, recovery, test mode), see [Execution model](../../concepts/execution-model.md).

## When linear mode fits

Linear mode works well when the task decomposes into a single path: research then write, extract then classify, or a fixed chat pipeline with two or three agents. You do not need conditional branches or parallel fan-out. When routing depends on step output, switch to [Graph workflows](graph-workflows.md).

Engine entry: `WorkflowEngine::execute_with_config` branches to `run_sorted_steps` for linear workflows.

## RCS structure

A linear workflow is a `WorkflowDefinition` with `execution_mode: "linear"`, a `steps` array, and `graph: null`. Each `StepDefinition` references an agent by UUID and carries an `order` field used only for sorting (not for graph routing).

| Field | Role |
|-------|------|
| `id`, `name` | Stable identity and trace labels |
| `execution_mode` | Must be `"linear"` |
| `steps` | List of steps; sorted by `order` ascending |
| `graph` | Absent or `null` for linear mode |
| `retry_policy` | Optional workflow-level retry (see [Retry and backoff](../reliability/retry-and-backoff.md)) |

Example two-step research pipeline:

```json
{
  "id": "00000000-0000-4000-8000-000000000001",
  "name": "research_pipeline",
  "execution_mode": "linear",
  "steps": [
    {
      "id": "00000000-0000-4000-8000-000000000010",
      "agent_id": "00000000-0000-4000-8000-000000000020",
      "order": 1,
      "fallback_step_id": null,
      "hitl": null
    },
    {
      "id": "00000000-0000-4000-8000-000000000011",
      "agent_id": "00000000-0000-4000-8000-000000000021",
      "order": 2,
      "fallback_step_id": null,
      "hitl": null
    }
  ],
  "graph": null,
  "external_bindings": null
}
```

Agents referenced by `agent_id` must appear in the run request `agents` array or be embedded in the workflow bundle. See [Defining agents](../agents-and-tools/defining-agents.md) for agent fields.

## State handoff between steps

The runtime passes prior step output to downstream agents according to each agent's `context` policy. Defaults include `include_prior_steps: "last"`, `include_run_input: true`, and `max_prior_step_chars: 4096`. Shared memory (`memory_type: "shared"`) lets agents read and write a namespace within the same run without relying on prompt injection alone.

## Python SDK example

After [installing the Python SDK](../../getting-started/install-and-build.md):

```python
from arcflow import Agent, Workflow

researcher = Agent(
    id="00000000-0000-4000-8000-000000000020",
    name="researcher",
    role="Research analyst",
    instructions="Gather facts on the topic.",
)
writer = Agent(
    id="00000000-0000-4000-8000-000000000021",
    name="writer",
    role="Writer",
    instructions="Summarize the research clearly.",
)

workflow = Workflow(
    name="research_pipeline",
    execution_mode="linear",
)
workflow.step(researcher, order=1)
workflow.step(writer, order=2)

result = workflow.run("Renewable energy trends in 2026")
print(result.output)
```

## Server API example

With [Server API quickstart](../../getting-started/quickstart-server-api.md) prerequisites (Postgres, migrations, API key):

