**Audience:** `[developer]`

# Execution traces

Execution traces answer operational questions: which steps ran, how long they took, whether tools and memory were used, and where failures occurred. ArcFlow exposes the same metadata through the SDK, HTTP API, CLI, and Postgres persistence. All trace payloads follow SEC-1 (counts and ids, not prompts or tool bodies).

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
