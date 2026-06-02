
# Execution traces

Execution traces answer operational questions: which steps ran, how long they took, whether tools and memory were used, and where failures occurred. ArcFlow exposes the same metadata through the SDK, HTTP API, CLI, and Postgres persistence. All trace payloads follow trace data policy (counts and ids, not prompts or tool bodies).

## Python: raw events on the result

After `workflow.run()`, read the flat event list:

```python
result = workflow.run("research quantum computing")
for event in result.trace_events:
 kind = event.get("event_kind") or event.get("kind")
 print(kind, event.get("step_id"), event.get("duration_ms"))
```

SDK exports may use `event_kind` as the field name in Python bindings; HTTP trace JSON uses `kind`.

Required lifecycle kinds for a healthy two-step stub run:

```python
kinds = {e.get("event_kind") or e.get("kind") for e in result.trace_events}
required = {"WorkflowStarted", "StepCompleted", "WorkflowCompleted"}
assert required.issubset(kinds)
```

## Python: structured trace()

`workflow.trace()` builds a `TraceResult` after `run()` on the same instance:

```python
result = workflow.run("research quantum computing")
trace = workflow.trace()

assert trace.run_id == result.run_id
assert len(trace) == result.step_count
print(trace.summary())

for step in trace:
 print(step.step_index, step.agent_name, step.duration_seconds)
 for tool in step.tools_called:
 print(" ", tool.tool_name, tool.status, tool.output_size_bytes)
 for mem in step.memory_operations:
 print(" ", mem.operation, mem.key, mem.hit)
```

`TraceResult` fields:

| Field | Meaning |
|-------|---------|
| `run_id` | UUID for this execution |
| `workflow_name` | Declared workflow name |
| `status` | `completed`, `failed`, or `partial` |
| `started_at` / `completed_at` | UTC timestamps |
| `total_duration_seconds` | Wall clock |
| `total_tokens_consumed` | Sum from step token usage |
| `steps` | Tuple of `StepTrace` |
| `warnings` | e.g. dropped events from store capacity |

`trace.summary()` returns a one-line human report. `trace.failed_step()` returns the first failed `StepTrace` if any.

## TypeScript

```typescript
const result = await workflow.run("research quantum computing");
const trace = workflow.trace();

console.log(trace.summary());
for (const step of trace.steps) {
 console.log(step.stepIndex, step.agentName, step.durationSeconds);
 for (const tool of step.toolsCalled) {
 console.log(" ", tool.toolName, tool.status);
 }
}
```

Raw events: `result.traceEvents` with `event_kind` per integration tests.

## Server API

Fetch persisted trace after a server run:

```bash
curl -s "http://localhost:8080/v1/runs/${RUN_ID}/trace" \
 -H "Authorization: Bearer ${ARCFLOW_SERVER_API_KEY}"
```

Response is an `ExecutionTrace` JSON document: ordered events plus optional aggregated step summaries depending on export version. Events are metadata-only.

Relay path for static sites:

```text
GET /v1/sites/{site_id}/runs/{run_id}/trace
```

## CLI

```bash
cargo run -p arcflow-cli -- trace RUN_ID --format json
cargo run -p arcflow-cli -- trace RUN_ID --tui
```

Requires access to the trace store (local SDK run or server-backed persistence).

## Annotated sample (three-step workflow with tool)

Metadata-only excerpt from a research → tool → write pipeline:

```json
[
 { "kind": "WorkflowStarted", "run_id": "r1", "workflow_name": "research_pipeline", "step_count": 3 },
 { "kind": "StepStarted", "run_id": "r1", "step_id": "s1", "step_index": 0, "agent_name": "researcher", "agent_role": "research" },
 { "kind": "ProviderRequestSent", "run_id": "r1", "step_id": "s1", "provider_id": "openai", "model_id": "gpt-4o-mini", "max_tokens": 1024, "prompt_size_bytes": 512 },
 { "kind": "ProviderResponseReceived", "run_id": "r1", "step_id": "s1", "provider_id": "openai", "model_id": "gpt-4o-mini", "tokens": { "input": 120, "output": 45, "total": 165 }, "latency_ms": 890 },
 { "kind": "StepCompleted", "run_id": "r1", "step_id": "s1", "step_index": 0, "duration_ms": 920, "output_size_bytes": 180 },
 { "kind": "StepStarted", "run_id": "r1", "step_id": "s2", "step_index": 1, "agent_name": "researcher", "agent_role": "research" },
 { "kind": "ToolCallStarted", "run_id": "r1", "step_id": "s2", "tool_name": "search_kb", "input_schema_hash": "a1b2c3..." },
 { "kind": "MemoryRetrieved", "run_id": "r1", "step_id": "s2", "agent_name": "researcher", "chunk_count": 5, "total_bytes": 8192 },
 { "kind": "ToolCallCompleted", "run_id": "r1", "step_id": "s2", "tool_name": "search_kb", "duration_ms": 210, "output_size_bytes": 8192 },
 { "kind": "StepCompleted", "run_id": "r1", "step_id": "s2", "step_index": 1, "duration_ms": 450, "output_size_bytes": 200 },
 { "kind": "StepStarted", "run_id": "r1", "step_id": "s3", "step_index": 2, "agent_name": "writer", "agent_role": "write" },
 { "kind": "StepCompleted", "run_id": "r1", "step_id": "s3", "step_index": 2, "duration_ms": 800, "output_size_bytes": 640 },
 { "kind": "WorkflowCompleted", "run_id": "r1", "duration_ms": 2200, "total_tokens": { "input": 400, "output": 180, "total": 580 } }
]
```

Reading guide:

- `prompt_size_bytes` and `output_size_bytes` replace raw text.
- `MemoryRetrieved` reports `chunk_count` and `total_bytes`, not chunk content.
- `ToolCallStarted` uses `input_schema_hash`, not argument JSON.

Full field catalog: [trace event reference](trace-event-reference.md).

## Postgres

When trace persistence is enabled, rows land in `arcflow_trace_events`. Operators may SQL-query by `run_id` for dashboards until Operator dashboard UI dashboard UI exits private repo CI.

## Related pages

- [Trace event reference](trace-event-reference.md) for every `TraceEventKind`
- [Trace data policy rules](sec-1-rules.md) for allowed and forbidden fields
- [Track A first workflow](../../tutorials/track-a-first-workflow.md) for hands-on trace verification
