# Graph Workflow Examples

## Problem

Many real automations are **not linear**. Teams need:

| Pattern | Real situation |
|---------|----------------|
| **Parallel search** | Product launch: query web + internal docs simultaneously, then merge |
| **ReAct loop** | Support agent iterates think, act, observe until answer or cap |
| **Reflection** | Content team drafts, critiques, revises until quality bar met |

Linear `step()` chains hide branching, fan-out, joins, and iteration limits. Graph mode models these explicitly.

## Examples in this directory

| Script | Problem | Graph features |
|--------|---------|----------------|
| [`launch_competitive_brief.py`](launch_competitive_brief.py) | Competitive intel for a launch brief | Fan-out, **join node** |
| [`support_react_loop.py`](support_react_loop.py) | Bounded tool loop for support tickets | Cycle, `max_iterations` |
| [`content_reflection_loop.py`](content_reflection_loop.py) | Blog quality gate | Conditional edge back to draft |

## Prerequisites

```bash
pip install -e sdk-python
```

## Run

```bash
python examples/graph/launch_competitive_brief.py
python examples/graph/support_react_loop.py
python examples/graph/content_reflection_loop.py
```

## Verify

- Each script prints `run_id` and `step_count > 0`
- `launch_competitive_brief`: synthesize step runs after both search branches
- `support_react_loop`: iteration bounded by `max_iterations(5)`
- `content_reflection_loop`: conditional edge `needs_more` on revise back to draft

## Production notes

- Graph recovery resume is partial (FP-1.01); design idempotent nodes
- Export graph as RCS JSON for server runs; see [capabilities reference](../../docs/_draft/ARCFLOW-FULL-CAPABILITIES-REFERENCE.md) §4.6
- VS Code extension visualizes graph definitions

## Related

- [education/](../education/), linear RAG
- [personal/weekly_blog_pipeline.py](../personal/weekly_blog_pipeline.py), linear alternative to reflection
