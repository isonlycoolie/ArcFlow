# 02 Session and shared memory


## Before you start

Read [01 Memory types overview](01-memory-types-overview.md) so `MemoryType` and `MemoryScope` are clear. This lesson uses only the stub provider. No Qdrant, Postgres, or API keys are required.

## Concept

Session and shared memory both live inside the runtime for the duration of a workflow run. When the run completes, that state is gone unless you also use persistent or vector backends elsewhere.

**Session memory** (`MemoryType.SESSION`) gives one agent a private key-value map for the current run. Use it for intermediate values the agent or its tools need within a single step loop: iteration counts, parsed flags, short-lived notes.

**Shared memory** (`MemoryType.SHARED`) exposes a namespace that multiple agents in the same workflow can read and write during one run. Use it when step two needs structured facts from step one and truncating prior step text through context policy would lose too much detail.

| Choice | Type | Scope | Namespace |
|--------|------|-------|-----------|
| One agent scratch pad | `SESSION` | `AGENT` | Optional but recommended for clarity |
| Two agents, same pipeline | `SHARED` | `WORKFLOW` | Required for predictable handoff keys |

Neither type talks to Qdrant. Trace events for reads and writes appear as `MemoryRead` and `MemoryWrite` with keys and hit flags, not chunk text (trace data policy).

## Example

Two agents share a workflow-scoped namespace. The first agent writes a summary key; the second agent is configured to read shared memory so the runtime can inject prior shared state into its context.

Save as `shared_memory_demo.py`:

```python
from arcflow import Agent, MemoryConfig, MemoryScope, MemoryType, Workflow

extractor = Agent(
 name="extractor",
 role="extractor",
 instructions="Extract three bullet facts from the user topic. Store them in shared memory under key 'facts'.",
 memory=MemoryConfig(
 MemoryType.SHARED,
 MemoryScope.WORKFLOW,
 namespace="handoff",
 ),
)

writer = Agent(
 name="writer",
 role="writer",
 instructions="Write a short paragraph using shared memory facts when available.",
 memory=MemoryConfig(
 MemoryType.SHARED,
 MemoryScope.WORKFLOW,
 namespace="handoff",
 ),
)

workflow = Workflow("shared-memory-demo")
workflow.step(extractor)
workflow.step(writer)

result = workflow.run("Solar panel efficiency improvements in 2024")
print(result.output)
print(f"status={result.status} steps={result.step_count}")
```

Session-only variant (single agent, agent-scoped scratch):

```python
from arcflow import Agent, MemoryConfig, MemoryScope, MemoryType, Workflow

planner = Agent(
 name="planner",
 role="planner",
 instructions="Plan a three-step outline for the topic.",
 memory=MemoryConfig(
 MemoryType.SESSION,
 MemoryScope.AGENT,
 namespace="scratch",
 ),
)

result = Workflow("session-demo").step(planner).run("Weekend hiking checklist")
print(result.output)
```

Run either script with `python shared_memory_demo.py`. Stub output is placeholder text, but the workflow completes and memory config serializes correctly to the Rust runtime.

## Verify

| Check | Expected |
|-------|----------|
| Script exits without exception | Yes |
| `result.status` | `"completed"` |
| `result.step_count` | `2` for the shared demo, `1` for the session demo |
| Shared demo uses same `namespace` on both agents | `"handoff"` on extractor and writer |

Inspect trace metadata after a run if you want to confirm memory events:

```python
kinds = {e.get("event_kind") for e in result.trace_events}
print(sorted(kinds))
```

You may see `MemoryWrite` or `MemoryRead` depending on runtime version and stub behavior. Absence of those events on stub-only runs is not a failure; the config wiring is what this lesson validates.

## Next

[03 Vector memory setup](03-vector-memory-setup.md) connects `MemoryType.VECTOR` to Qdrant through `ARCFLOW_QDRANT_URL` and namespace alignment.
