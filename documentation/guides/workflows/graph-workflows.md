**Audience:** `[developer]`

# Graph workflows

Graph execution runs workflows as a directed acyclic graph (DAG). Nodes map to steps, edges route by condition, and join nodes synchronize parallel branches. Use graph mode when step output determines which path to take, when you need parallel fan-out, or when multiple branches must merge before continuing.

Prerequisites: understand [Linear workflows](linear-workflows.md) and [Execution model](../../concepts/execution-model.md). Graph workflows still use the same agent loop, tools, and trace machinery; only scheduling differs.

## RCS structure

Set `execution_mode: "graph"` and supply a `graph` object alongside `steps`. Steps are referenced from graph nodes via `step_ref` (a step id, not an agent id).

| Graph field | Purpose |
|-------------|---------|
| `entry_node` | Node id where execution starts |
| `max_iterations` | Loop guard against infinite cycles |
| `nodes` | Each node has `id`, `step_ref`, optional `inputs`, optional `outputs` |
| `edges` | `from`, optional `to`, optional `condition` |
| `join_nodes` | Nodes that wait for all `wait_for` branch ids to complete |

### Node outputs and graph state

`GraphNode.outputs` maps keys written into graph state JSON. Downstream agents receive this state through `exec_config.initial_state` and the runtime map. Example: a classify node can write `{ "route": "category" }` so later routing or prompts can reference structured state.

### Edge routing

After each node completes, trimmed step output may match an edge `condition` to choose the next branch. When `to` is `null`, that branch terminates. Unconditional edges omit `condition`.

### Join nodes

A join node runs only when every id in `wait_for` appears in the completed set. Typical pattern: parallel billing and technical paths both feed a summarize node.

### Parallel fan-out

Multiple edges from one node trigger parallel execution. The scheduler uses parallel executor mode. Join nodes reconcile branches before the next single-path step.

## Complete example: support router

```json
{
  "id": "00000000-0000-4000-8000-000000000002",
  "name": "support_router",
  "execution_mode": "graph",
  "steps": [
    { "id": "s-classify", "agent_id": "a-classify", "order": 1 },
    { "id": "s-billing", "agent_id": "a-billing", "order": 2 },
    { "id": "s-tech", "agent_id": "a-tech", "order": 3 },
    { "id": "s-merge", "agent_id": "a-summarize", "order": 4 }
  ],
  "graph": {
    "entry_node": "n-classify",
    "max_iterations": 20,
    "nodes": [
      { "id": "n-classify", "step_ref": "s-classify", "outputs": { "route": "category" } },
      { "id": "n-billing", "step_ref": "s-billing" },
      { "id": "n-tech", "step_ref": "s-tech" },
      { "id": "n-merge", "step_ref": "s-merge" }
    ],
    "edges": [
      { "from": "n-classify", "to": "n-billing", "condition": "billing" },
      { "from": "n-classify", "to": "n-tech", "condition": "technical" },
      { "from": "n-billing", "to": "n-merge" },
      { "from": "n-tech", "to": "n-merge" }
    ],
    "join_nodes": [
      { "id": "n-merge", "wait_for": ["n-billing", "n-tech"] }
    ]
  }
}
```

The classify agent should emit output that matches edge conditions (for example the literal token `billing` or `technical` after trimming). Mismatch behavior depends on scheduler rules; validate routing in [Validation and testing](validation-and-testing.md) with test mode.

## Engine behavior

Entry point: `WorkflowEngine::execute_with_config` calls `run_graph_loop` in `workflow/graph/scheduler.rs`.

Checkpoints: after each node, when `recovery_enabled` is true, `persist_graph_checkpoint` upserts `current_node_id` and `graph_iteration_count` to Postgres.

**Graph recovery resume (FP-1.01):** checkpoint schema and persist path exist. Full resume dispatch from a mid-graph checkpoint after failure is **partial**. Do not rely on graph resume in production until FP-1.01 closes. Linear recovery is complete; see [Recovery and resume](../reliability/recovery-and-resume.md) and [Maturity and known gaps](../../concepts/maturity-and-known-gaps.md).

## Graph-specific trace events

In addition to standard step events, graph runs may emit:

| Event | Meaning |
|-------|---------|
| `GraphNodeStarted` | Scheduler dispatched a graph node |
| `GraphNodeCompleted` | Node finished; edges evaluated next |
| `GraphIterationLimitReached` | `max_iterations` exceeded; run fails |

Example excerpt for a billing branch:

```json
[
  { "kind": "GraphNodeStarted", "run_id": "r1", "node_id": "n-classify" },
  { "kind": "StepStarted", "run_id": "r1", "step_id": "s-classify", "step_index": 0, "agent_name": "classifier" },
  { "kind": "StepCompleted", "run_id": "r1", "step_id": "s-classify", "step_index": 0, "duration_ms": 400 },
  { "kind": "GraphNodeCompleted", "run_id": "r1", "node_id": "n-classify" },
  { "kind": "GraphNodeStarted", "run_id": "r1", "node_id": "n-billing" },
  { "kind": "StepStarted", "run_id": "r1", "step_id": "s-billing", "step_index": 1, "agent_name": "billing" }
]
```

## TypeScript SDK example

```typescript
import { Workflow, Agent } from "@arcflow/sdk";

const classify = new Agent({
  id: "a-classify",
  name: "classifier",
  role: "Router",
  instructions: "Reply with exactly one word: billing or technical.",
});

const workflow = new Workflow({
  name: "support_router",
  execution_mode: "graph",
  steps: [
    { id: "s-classify", agentId: "a-classify", order: 1 },
    { id: "s-billing", agentId: "a-billing", order: 2 },
    { id: "s-tech", agentId: "a-tech", order: 3 },
    { id: "s-merge", agentId: "a-summarize", order: 4 },
  ],
  graph: {
    entryNode: "n-classify",
    maxIterations: 20,
    nodes: [
      { id: "n-classify", stepRef: "s-classify", outputs: { route: "category" } },
      { id: "n-billing", stepRef: "s-billing" },
      { id: "n-tech", stepRef: "s-tech" },
      { id: "n-merge", stepRef: "s-merge" },
    ],
    edges: [
      { from: "n-classify", to: "n-billing", condition: "billing" },
      { from: "n-classify", to: "n-tech", condition: "technical" },
      { from: "n-billing", to: "n-merge" },
      { from: "n-tech", to: "n-merge" },
    ],
    joinNodes: [{ id: "n-merge", waitFor: ["n-billing", "n-tech"] }],
  },
  agents: [classify],
});

const result = await workflow.run("I was charged twice on my invoice.");
```

See [TypeScript quickstart](../../getting-started/quickstart-typescript.md) for setup.

## exec_config for graph runs

Pass initial graph state when resuming or seeding downstream context:

```json
{
  "recovery_enabled": true,
  "workflow_timeout_secs": 600,
  "initial_state": {
    "observation": "prior ticket context from CRM"
  }
}
```

## Examples in the repo

Working samples live under `examples/graph/`. Verify trace order matches the branch you expect for each input class.

## Related pages

- [Linear workflows](linear-workflows.md)
- [Step fallbacks](step-fallbacks.md) for per-node resilience
- [The RCS contract](../../concepts/the-rcs-contract.md) (GraphDefinition, GraphNode, GraphEdge, JoinNode)
- [Validation and testing](validation-and-testing.md)

## Source

Derived from [ARCFLOW-FULL-CAPABILITIES-REFERENCE.md](../../../docs/_draft/ARCFLOW-FULL-CAPABILITIES-REFERENCE.md) §4.3, §4.6; Appendix A (GraphDefinition); FP-1.01 graph recovery note.
