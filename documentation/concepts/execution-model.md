
# Execution model

ArcFlow runs workflows in one of two execution modes: **linear** (ordered steps) or **graph** (DAG with conditional routing and joins). Both modes share the same agent execution loop, trace events, retry and timeout machinery, and terminal `ExecutionStatus` values. Mode selection is a field on the workflow specification workflow definition, not a separate product.

Engine entry is `WorkflowEngine::execute_with_config`, which branches to `run_sorted_steps` for linear workflows or `run_graph_loop` in `workflow/graph/scheduler.rs` for graph workflows.

## Linear execution

Linear mode sets `execution_mode: "linear"`. Steps are sorted by the numeric `order` field and run sequentially. Output from one step can flow to the next via agent `context` policy (`include_prior_steps`, `include_run_input`, `max_prior_step_chars`, default 4096).

Typical use cases: research pipelines, extract-then-summarize flows, single-path chat with one or more agents in fixed order.

Example:

```json
{
 "execution_mode": "linear",
 "steps": [
 { "id": "s1", "agent_id": "a-research", "order": 1 },
 { "id": "s2", "agent_id": "a-write", "order": 2 }
 ]
}
```

Optional `fallback_step_id` on a step routes to a backup agent on primary failure (`StepFallbackActivated` in trace). Step-level and workflow-level retry policies live in the workflow specification and `exec_config`.

When `recovery_enabled` is true, linear runs persist progress to Postgres. On resume, the engine continues from the last committed step index (`WorkflowRecoveryStarted`, `WorkflowRecoveryCompleted` trace events).

## Graph execution

Graph mode sets `execution_mode: "graph"` and supplies a `graph` object alongside `steps` (steps are referenced by nodes via `step_ref`).

| Concept | Behavior |
|---------|----------|
| `entry_node` | Where execution starts |
| `nodes` | Map graph node id to step; optional `outputs` keys write into graph state JSON |
| `edges` | `from`, optional `to`, optional `condition` matched against trimmed step output |
| `to: null` | Terminates that branch |
| `join_nodes` | Node runs only when all `wait_for` branch ids are in the completed set |
| `max_iterations` | Guards infinite loops |
| Parallel fan-out | Multiple edges from one node; scheduler uses parallel executor mode |

After each node completes, the scheduler may evaluate edge conditions to pick the next branch. Downstream agents receive graph state through `exec_config.initial_state` and the runtime map.

Checkpoints: when recovery is enabled, each node triggers `persist_graph_checkpoint` with `current_node_id` and `graph_iteration_count`. **Full resume dispatch from graph checkpoints is partial (Graph recovery resume):** schema and persist path exist; resuming mid-graph after failure is not production-complete.

Example router pattern:

```json
{
 "execution_mode": "graph",
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
 "join_nodes": [{ "id": "n-merge", "wait_for": ["n-billing", "n-tech"] }]
 }
}
```

Graph-specific trace kinds include `GraphNodeStarted`, `GraphNodeCompleted`, and `GraphIterationLimitReached`.

## Agent step loop (both modes)

Within each step, the default tool mode is `llm_select`: the provider receives tool schemas, the model may emit tool calls, the runtime validates input against JSON Schema, and results feed back until text output or `max_iterations`. Trace sequence for a tool call: `ToolCallStarted` through `ToolCallCompleted` or `ToolCallFailed`, with provider events in between when the LLM selects tools.

Memory reads and vector retrieval emit trace data policy safe events (`MemoryRetrieved` records `chunk_count` and `total_bytes` only).

## Run state machine

`ExecutionStatus` values: `Pending`, `Running`, `Completed`, `Failed`, `Retrying`, `Cancelled`, `Interrupted`.

### Linear lifecycle

After `POST /v1/runs` or SDK `run()`:

```
Pending → Running → Completed
 └→ Failed → (retry?) → Retrying → Running
 └→ Interrupted (HITL) → (approve) → Running
 └→ Cancelled
```

### Human-in-the-loop

A step with `hitl` config requires `recovery_enabled`. When the gate fires, status becomes `Interrupted`. `GET /v1/runs/{id}` returns interrupt payload with `approval_key` and metadata (no trace data policy violations). Approver calls `POST /v1/runs/{id}/approve/{approval_key}` with `{ "approved": true, "data": {} }` or rejection with `approved: false`. Approve resumes; reject yields `HumanRejected`; timeout yields `HumanTimeout`.

### Recovery

With `recovery_enabled`, failed or interrupted runs can persist `RecoveryState` in Postgres: `completed_steps`, `failed_at_step_index`, and for graphs `current_node_id`, `graph_iteration_count`, `pending_join`. Linear resume is production-ready. Graph resume dispatch remains partial (Graph recovery resume).

## Streaming vs polling

SDKs support `run_stream()` with token deltas at the SDK layer; engine emits `StreamChunkReceived` and `TokenEmitted` (counts and sizes only). Server SSE at `GET /v1/runs/{id}/events` is **not implemented (streaming deferred)**. Server and static browser clients poll run status or trace.

## Test mode

`exec_config.test` with per-step `fail_times`, `output`, and `then_output` drives deterministic runs without live LLM for CI. Useful for retry and fallback traces without provider cost.

## Related pages

- [Workflow specification](the-rcs-contract.md) for workflow and agent types
- [Trace data policy](sec-1-and-data-safety.md) for trace contents
- [Maturity and known gaps](maturity-and-known-gaps.md) for Graph recovery resume and 
