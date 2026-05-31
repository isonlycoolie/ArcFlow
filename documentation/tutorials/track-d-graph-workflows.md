# Track D: Graph workflows


Track D builds conditional-branch graph workflows with routing and join nodes. You verify correct branch execution order through graph trace events, not only final output text.

## Goal

Build a conditional-branch graph workflow with routing and a join node. Verify correct branch execution order via trace metadata (`GraphNodeStarted`, `GraphNodeCompleted`).

## Prerequisites

| Item | Required |
|------|----------|
| [Track A](track-a-first-workflow.md) | Linear workflow familiarity |
| Python SDK | Built |
| Provider | Stub default |
| Primary examples | [Graph routing example](../examples/graph-routing.md) |
| Scripts | [`parallel_search.py`](../examples/graph-routing.md), [`reflection_loop.py`](../examples/graph-routing.md) |
| Guide | [Graph workflows](../guides/workflows/graph-workflows.md) |

Postgres optional for embedded SDK; required for server recovery paths. Graph resume from checkpoint is partial (FP-1.01).

## Step 1: Parallel fan-out and join

Run the canonical join sample:

```bash
python examples/graph/parallel_search.py
```

Study wiring:

```python
wf = (
    Workflow("parallel_search", graph=True)
    .max_iterations(10)
    .node("router", router)
    .node("search_web", search_web)
    .node("search_docs", search_docs)
    .node("synthesize", synthesize)
    .set_entry("router")
    .add_edge("router", "search_web")
    .add_edge("router", "search_docs")
    .add_edge("search_web", "synthesize")
    .add_edge("search_docs", "synthesize")
    .join_node("synthesize", ["search_web", "search_docs"])
    .add_edge("synthesize", None)
)
result = wf.run("query=ArcFlow graph execution parallel fan-out")
```

## Step 2: Conditional loop (optional second exercise)

```bash
python examples/graph/reflection_loop.py
```

Observe `max_iterations(3)` guarding the revise-to-draft loop.

## Step 3: Extract graph trace order

```python
starts = [
    (e.get("node_id"), e.get("iteration"))
    for e in result.trace_events
    if e.get("event_kind") == "GraphNodeStarted"
]
print("node start order:", starts)
```

For parallel search, both `search_web` and `search_docs` should appear before the final `synthesize` completion. Order between parallel branches may vary; join ensures synthesize waits for both.

## Step 4: Verification checklist

| Check | Expected |
|-------|----------|
| `result.status` | `completed` |
| `GraphNodeStarted` | One entry per executed node (plus iterations in loop sample) |
| Join behavior | `synthesize` after both search nodes in parallel exercise |
| `WorkflowCompleted` | Present |
| `max_iterations` | Loop sample does not exceed guard |

## Expected output

```
run_id=<uuid> steps=<n>
```

Pass criteria are structural: graph events and join ordering, not exact LLM text.

## Trace events you should see

| Event kind | When |
|------------|------|
| `WorkflowStarted` | Graph run begins |
| `GraphNodeStarted` | Each node starts |
| `GraphNodeCompleted` | Each node finishes |
| `StepStarted` / `StepCompleted` | Per underlying step |
| `WorkflowCompleted` | Success |

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| Synthesize runs early | Join misconfigured | Use `.join_node("synthesize", ["search_web", "search_docs"])` |
| Loop never exits | Condition always matches | Add terminating edge with `to=None` |
| Missing graph events | `graph=True` omitted | Set on Workflow constructor |
| Exceeded iterations error | Guard too low | Increase `max_iterations` or fix routing |

## What you learned

Track D covers scheduler semantics: entry nodes, conditional edges, parallel fan-out, join preconditions, and iteration guards. These patterns underpin support routers and multi-branch automation in production.

## Next tracks

| Track | Focus |
|-------|-------|
| E | HITL and external callbacks |
| Level 2 cert | Graph plus RAG plus HITL combined workflow |
