# Understanding ArcFlow traces

ArcFlow records **metadata-only** execution traces for every workflow run. Traces help you answer: which steps ran, how long they took, whether tools or memory were used, and where a run failed, without exposing user payloads.

## Python: `workflow.trace()`

After `workflow.run()` returns, call `workflow.trace()` on the same instance:

```python
from arcflow import Agent, Workflow

wf = Workflow("research")
wf.step(Agent(name="researcher", role="researcher", instructions="Research the topic."))
result = wf.run("renewable energy trends")
trace = wf.trace()

print(trace.summary())
print(trace.status)          # completed | failed | partial
print(len(trace.steps))
failed = trace.failed_step() # None when successful
```

`TraceResult` is immutable. Timing is in **seconds** at the Python boundary. Token counts and tool/memory metadata are included; instructions, workflow input, tool output, and memory values are never included (SEC-1).

## CLI: `arcflow trace`

When a Python process is not available, use the Rust CLI against the in-process store (same process that executed the workflow):

```bash
cargo run -p arcflow-cli -- trace <run-id> --format human
cargo run -p arcflow-cli -- trace <run-id> --format json --verbose
```

See [cli-reference.md](cli-reference.md) for flags and exit codes.

## OTLP export (optional)

Set `ARCFLOW_OTLP_ENDPOINT` to export metadata spans after run completion. Export is best-effort and never blocks workflow execution.

## Limits

- `MAX_TRACE_EVENTS_PER_RUN` = 10,000 per run
- `MAX_CONCURRENT_TRACES` = 100 completed runs in the process store

## Related contracts

- [Event reference](event-reference.md)
- [Debugging guide](debugging-guide.md)
- [Trace events (normative)](../../trace-events-normative.md)
