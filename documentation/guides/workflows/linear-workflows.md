
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

```json
{
  "workflow": {
    "id": "00000000-0000-4000-8000-000000000001",
    "name": "research_pipeline",
    "execution_mode": "linear",
    "steps": [
      { "id": "00000000-0000-4000-8000-000000000010", "agent_id": "00000000-0000-4000-8000-000000000020", "order": 1 },
      { "id": "00000000-0000-4000-8000-000000000011", "agent_id": "00000000-0000-4000-8000-000000000021", "order": 2 }
    ]
  },
  "agents": [
    {
      "id": "00000000-0000-4000-8000-000000000020",
      "name": "researcher",
      "role": "Research analyst",
      "instructions": "Gather facts on the topic."
    },
    {
      "id": "00000000-0000-4000-8000-000000000021",
      "name": "writer",
      "role": "Writer",
      "instructions": "Summarize the research clearly."
    }
  ],
  "input": "Renewable energy trends in 2026",
  "exec_config": {
    "recovery_enabled": true,
    "workflow_timeout_secs": 600,
    "step_timeout_secs": 120
  }
}
```

POST to `/v1/runs` with `Authorization: Bearer <ARCFLOW_SERVER_API_KEY>`. Poll `GET /v1/runs/{run_id}` until status is terminal.

## Trace events to expect

A successful linear run typically emits:

```json
[
  { "kind": "WorkflowStarted", "run_id": "r1", "workflow_name": "research_pipeline", "step_count": 2 },
  { "kind": "StepStarted", "run_id": "r1", "step_id": "00000000-0000-4000-8000-000000000010", "step_index": 0, "agent_name": "researcher", "agent_role": "Research analyst" },
  { "kind": "StepCompleted", "run_id": "r1", "step_id": "00000000-0000-4000-8000-000000000010", "step_index": 0, "duration_ms": 920, "output_size_bytes": 180 },
  { "kind": "StateCommitted", "run_id": "r1", "step_id": "00000000-0000-4000-8000-000000000010", "committed_step_count": 1 },
  { "kind": "StepStarted", "run_id": "r1", "step_id": "00000000-0000-4000-8000-000000000011", "step_index": 1, "agent_name": "writer", "agent_role": "Writer" },
  { "kind": "StepCompleted", "run_id": "r1", "step_id": "00000000-0000-4000-8000-000000000011", "step_index": 1, "duration_ms": 850, "output_size_bytes": 240 },
  { "kind": "WorkflowCompleted", "run_id": "r1", "duration_ms": 1800 }
]
```

Traces are metadata-only per [SEC-1 and data safety](../../concepts/sec-1-and-data-safety.md).

## Recovery

When `exec_config.recovery_enabled` is true, linear progress persists to Postgres after each committed step. On resume, the engine continues from the last committed step index (`WorkflowRecoveryStarted`, `WorkflowRecoveryCompleted`). Linear recovery is production-ready. See [Recovery and resume](../reliability/recovery-and-resume.md).

## Optional resilience

| Mechanism | Config location | Guide |
|-----------|-----------------|-------|
| Step fallback | `fallback_step_id` on step | [Step fallbacks](step-fallbacks.md) |
| Retry | `exec_config.retry`, step `retry_policy` | [Retry and backoff](../reliability/retry-and-backoff.md) |
| Timeouts | `workflow_timeout_secs`, `step_timeout_secs` | [Timeouts](../reliability/timeouts.md) |
| Validation | Schema + engine validate | [Validation and testing](validation-and-testing.md) |

## Related pages

- [Graph workflows](graph-workflows.md) for conditional routing and joins
- [The RCS contract](../../concepts/the-rcs-contract.md) for type definitions
- [Python quickstart](../../getting-started/quickstart-python.md) and [TypeScript quickstart](../../getting-started/quickstart-typescript.md)
