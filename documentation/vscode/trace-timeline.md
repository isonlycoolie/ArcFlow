
# Trace timeline (VS Code)

Inspect trace data policy execution traces as a chronological timeline in VS Code. Useful after failed runs, retry storms, or HITL interrupts.

## Open trace timeline

1. Export or save trace JSON as `run-abc.arcflow.trace.json`.
2. Command palette: **ArcFlow: View Trace Timeline** (`arcflow.viewTraceTimeline`).
3. Or open a `*.arcflow.trace.json` file and use the editor title timeline button.

## Obtaining trace JSON

From server:

```bash
curl -s "http://localhost:8080/v1/runs/RUN_ID/trace" \
 -H "Authorization: Bearer dev-secret" \
 > run.arcflow.trace.json
```

From CLI:

```bash
arcflow trace RUN_ID --format json > run.arcflow.trace.json
```

Rename to `*.arcflow.trace.json` so VS Code activates the ArcFlow trace language mode.

## Timeline content

Each event shows PascalCase `kind` and metadata fields only (trace data policy). Example sequence for a two-step linear workflow:

```text
t+0ms WorkflowStarted step_count=2
t+12ms StepStarted step_id=...
t+45ms ProviderRequestSent provider=stub, input_tokens=10
t+48ms ProviderResponseReceived output_tokens=25
t+50ms StepCompleted
t+55ms StepStarted step_id=...
t+90ms StepCompleted
t+92ms WorkflowCompleted duration_ms=92
```

Failed run with retry:

```text
t+0ms WorkflowStarted
t+20ms StepStarted
t+100ms ProviderError retryable=true
t+105ms RetryAttempted attempt=1
t+1200ms StepCompleted
t+1210ms WorkflowCompleted
```

HITL interrupt:

```text
...
t+500ms WorkflowInterrupted approval_key=manager_signoff
```

No prompt text, tool arguments, or memory chunk content appears in the timeline.

## TUI alternative

Terminal timeline without VS Code:

```bash
arcflow trace RUN_ID --tui
```

Or fetch from server:

```bash
arcflow trace RUN_ID --server http://localhost:8080 --tui
```

## Field reference

Full event catalog: [Trace event reference](../guides/observability/trace-event-reference.md).

Normative contract: [Trace events (normative)](../contracts/trace-events-normative.md).

## Related pages

- [Trace command](../cli/trace.md)
- [Execution traces](../guides/observability/execution-traces.md)
- [Overview](overview.md)
