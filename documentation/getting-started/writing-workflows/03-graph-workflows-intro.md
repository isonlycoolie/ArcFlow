# 03 Graph workflows intro

**Audience:** `[developer]`

## Before you start

Complete [01 Linear pipelines](01-linear-pipelines.md). You should understand `Workflow.step()` and when a linear order is not enough (for example, routing to different specialists based on classification output).

## Concept

**Graph mode** runs agents as nodes in a directed graph instead of a fixed list. You choose the next node based on edge conditions, run branches in parallel, and merge branches at **join** nodes.

Enable graph mode with `Workflow("name", graph=True)`. Use these methods instead of `step()`:

| Method | Purpose |
|--------|---------|
| `node(id, agent)` | Register a graph node backed by an agent |
| `set_entry(id)` | Where execution starts (first registered node is the default entry) |
| `add_edge(from_id, to_id, condition=...)` | Route to the next node; `to_id=None` ends a branch |
| `join_node(id, wait_for=[...])` | Wait for listed branch nodes before continuing |
| `max_iterations(n)` | Guard against infinite loops (default 100) |

You cannot mix `step()` and `node()` on the same workflow. Graph and linear builders are mutually exclusive.

After a node completes, trimmed step output may match an edge `condition` string to pick the next branch. Validate routing in traces (`GraphNodeStarted`, `GraphNodeCompleted`) rather than guessing from final text alone.

### Recovery note (FP-1.01)

Graph **execution** is production-ready: conditional edges, join nodes, and parallel fan-out work today. **Resume from checkpoint after failure** is partial (**FP-1.01**). The runtime persists checkpoint fields (`current_node_id`, `graph_iteration_count`, `pending_join`) for observability, but mid-graph resume dispatch is incomplete. Plan linear recovery patterns for critical SLAs until FP-1.01 closes. See [Maturity and known gaps](../../concepts/maturity-and-known-gaps.md).

## Example

A minimal classifier routes to billing or technical handling:

Save as `graph_intro.py`:

```python
from arcflow import Agent, Workflow

classifier = Agent(
    name="classifier",
    role="Classifier",
    instructions=(
        "Classify the ticket. Reply with exactly one word: billing or technical."
    ),
)

billing = Agent(
    name="billing",
    role="Billing",
    instructions="Draft a billing response.",
)

technical = Agent(
    name="technical",
    role="Technical",
    instructions="Draft a technical support response.",
)

workflow = (
    Workflow("support-router", graph=True)
    .max_iterations(10)
    .node("classify", classifier)
    .node("billing", billing)
    .node("technical", technical)
    .set_entry("classify")
    .add_edge("classify", "billing", condition="billing")
    .add_edge("classify", "technical", condition="technical")
    .add_edge("billing", None)
    .add_edge("technical", None)
)

result = workflow.run("Customer asks about invoice line items")

print(result.output[:200], "..." if len(result.output) > 200 else "")
print(f"status={result.status}")

starts = [
    e.get("node_id")
    for e in result.trace_events
    if e.get("event_kind") == "GraphNodeStarted"
]
print("nodes started:", starts)
```

Run:

```bash
python graph_intro.py
```

Stub output may not always emit the exact condition token `billing` or `technical`; for routing drills, use [Track D](../../tutorials/track-d-graph-workflows.md) or test mode once you need deterministic branch proofs.

## Verify

| Check | Expected |
|-------|----------|
| `result.status` | `"completed"` on happy path |
| Trace events | At least one `GraphNodeStarted` |
| Mixed API rejected | `Workflow(graph=True).step(agent)` raises `WorkflowConfigurationError` |
| Linear API rejected | `Workflow().node("a", agent)` raises `WorkflowConfigurationError` |

Mixed API check:

```python
from arcflow import Agent, Workflow
from arcflow.exceptions import WorkflowConfigurationError

agent = Agent(name="a", role="a", instructions="Run.")
try:
    Workflow("g", graph=True).node("a", agent).step(agent)
except WorkflowConfigurationError as err:
    print(err)
```

## Next

[04 Testing with stub responses](04-testing-with-stub-responses.md) shows how to pin step outputs for CI without live model calls.

## Source

`sdk-python/arcflow/workflow.py` (`node`, `add_edge`, `join_node`, `set_entry`); `sdk-python/tests/unit/test_graph_workflow.py`; [Graph workflows](../../guides/workflows/graph-workflows.md); [FP-1.01 maturity note](../../concepts/maturity-and-known-gaps.md).
