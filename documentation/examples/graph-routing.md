# Graph routing example

**Audience:** `[developer]`

This walkthrough runs graph workflows with conditional edges, loops, and parallel join nodes. You verify branch order through trace metadata (`GraphNodeStarted`, `GraphNodeCompleted`). Scripts live under [`examples/graph/`](../../examples/graph/).

## What this example demonstrates

Graph mode (`Workflow(..., graph=True)`) schedules nodes instead of a flat step list. Three patterns ship in the repo:

| Script | Pattern |
|--------|---------|
| [`reflection_loop.py`](../../examples/graph/reflection_loop.py) | Conditional loop with `max_iterations` guard |
| [`parallel_search.py`](../../examples/graph/parallel_search.py) | Fan-out to two branches, join at synthesize |
| [`react_agent.py`](../../examples/graph/react_agent.py) | ReAct-style tool loop in graph form |

This document focuses on reflection and parallel search; open `react_agent.py` for tool-loop routing.

## Prerequisites

| Item | Required |
|------|----------|
| Python SDK | Built and importable |
| Provider | Stub default |
| Postgres | Optional for embedded SDK; required for recovery-heavy server paths |
| Reading | [Graph workflows](../guides/workflows/graph-workflows.md) |
| Tutorial track | [Track D](../tutorials/track-d-graph-workflows.md) |

Note: graph checkpoint resume remains partial (FP-1.01). Linear recovery is complete; do not rely on mid-graph resume in production.

## Step 1: Reflection loop with conditional edge

Run:

```bash
python examples/graph/reflection_loop.py
```

Core graph wiring from the script:

```python
wf = (
    Workflow("reflection_loop", graph=True)
    .max_iterations(3)
    .node("draft", draft)
    .node("critique", critique)
    .node("revise", revise)
    .set_entry("draft")
    .add_edge("draft", "critique")
    .add_edge("critique", "revise")
    .add_edge("revise", "draft", condition="needs_more")
    .add_edge("revise", None)
)
result = wf.run("topic=Benefits of local-first agent runtimes")
```

The `needs_more` condition routes back to draft when step output matches; termination uses `add_edge("revise", None)`.

## Step 2: Parallel fan-out and join

Run:

```bash
python examples/graph/parallel_search.py
```

Join configuration waits for both search branches:

```python
.join_node("synthesize", ["search_web", "search_docs"])
```

Expected: router runs first, both search nodes run, synthesize runs after join preconditions are met.

## Step 3: Verify trace branch order

```python
node_events = [
    e for e in result.trace_events
    if e.get("event_kind") in ("GraphNodeStarted", "GraphNodeCompleted")
]
for e in node_events:
    print(e.get("event_kind"), e.get("node_id"))
```

For parallel search, confirm `search_web` and `search_docs` both appear before final `synthesize` completion events.

## Expected output

Reflection loop:

```
run_id=<uuid> steps=<n>
```

Parallel search prints similar `run_id` and `steps` lines. Step count reflects graph iterations, not only node count. Values vary with routing and stub output.

Pass criteria:

| Check | Expected |
|-------|----------|
| `result.status` | `completed` |
| `result.run_id` | UUID |
| Graph trace kinds | Present for each executed node |
| Parallel join | Synthesize after both branches in trace order |

## Trace events you should see

| Event kind | When |
|------------|------|
| `WorkflowStarted` | Graph run begins |
| `GraphNodeStarted` | Each node begins (includes iteration metadata) |
| `GraphNodeCompleted` | Node finishes |
| `StepStarted` / `StepCompleted` | Underlying step execution per node |
| `WorkflowCompleted` | Terminal success |

Conditional re-entry adds repeated `GraphNodeStarted` for the same node id across iterations until exit or `max_iterations`.

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| Infinite loop or early exit | Condition string mismatch | Ensure agent output trim matches edge `condition` values |
| Join never fires | Missing `join_node` or wrong branch ids | Match `wait_for` ids to node names in parallel_search pattern |
| `WorkflowExecutionError: max iterations` | Loop guard hit | Increase `max_iterations` or fix termination edge |
| No graph events in trace | Old SDK build | Rebuild; confirm `graph=True` on Workflow |

## Related

| Resource | Link |
|----------|------|
| Tutorial track | [Track D](../tutorials/track-d-graph-workflows.md) |
| Graph guide | [Graph workflows](../guides/workflows/graph-workflows.md) |
| Maturity note | [FP-1.01 graph resume](../../concepts/maturity-and-known-gaps.md) |

**Source:** [`examples/graph/reflection_loop.py`](../../examples/graph/reflection_loop.py), [`examples/graph/parallel_search.py`](../../examples/graph/parallel_search.py), [`examples/graph/react_agent.py`](../../examples/graph/react_agent.py); capabilities reference §25, §28 Track D; [graph workflows](../guides/workflows/graph-workflows.md).
