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

