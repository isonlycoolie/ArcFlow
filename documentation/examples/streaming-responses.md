# Streaming responses example


This walkthrough consumes incremental stream events from the SDK during workflow execution. You iterate `run_stream()` (Python) or `runStream()` (TypeScript) and handle token, step start, and step complete events. Server-side SSE remains deferred (streaming deferred); this is in-process SDK streaming only.

Scripts: [Streaming responses walkthrough](../examples/streaming-responses.md), [Streaming responses walkthrough](../examples/streaming-responses.md).

## What this example demonstrates

Streaming exposes partial output and step lifecycle events before the final `WorkflowResult` is assembled. Useful for CLI chat UX and local tooling. Both language samples use the stub provider so no API keys are required.

## Prerequisites

| Item | Python | TypeScript |
|------|--------|------------|
| SDK built | `maturin develop` in `sdk-python/` | `npm run build` in `sdk-typescript/` |
| API keys | Not required (stub) | Not required (stub) |
| Async runtime | `asyncio` (stdlib) | Node 20+ with `tsx` or compiled output |
| Reading | [SDK streaming](../guides/streaming/sdk-streaming.md) | Same |

## Step 1: Python streaming run

```bash
python examples/streaming/chat_stream.py
```

Core loop:

```python
async for event in wf.run_stream("Hello from ArcFlow"):
 if event.type == "token":
 print(event.text, end="", flush=True)
 elif event.type == "step_start":
 print(f"\n[step start: {event.step_id}]")
 elif event.type == "step_complete":
 print(f"[step complete: {event.step_id} in {event.duration_ms}ms]")
```

## Step 2: TypeScript streaming run

From repository root after TypeScript SDK build:

```bash
npx tsx examples/streaming/chat_stream.ts
```

Equivalent pattern:

```typescript
for await (const event of wf.runStream("Hello from ArcFlow")) {
 switch (event.type) {
 case "token":
 process.stdout.write(event.text);
 break;
 case "step_start":
 process.stdout.write(`\n[step start: ${event.step_id}]\n`);
 break;
 case "step_complete":
 process.stdout.write(
 `[step complete: ${event.step_id} in ${event.duration_ms}ms]\n`,
 );
 break;
 }
}
```

## Step 3: Verify event count

Both scripts raise if zero events arrive. Expect at least one `step_start` and one `step_complete` on the stub path, plus token events when the provider emits them.

## Expected output

```
Streaming events:

[step start: <step_id>]
<token text may appear here>
[step complete: <step_id> in <ms>ms]

Done.
```

Exact token text varies. Pass criteria: non-zero event count, no uncaught exceptions, terminal `Done.` line.

## Trace events you should see

Streaming complements but does not replace trace export. After the stream finishes, call `workflow.run()` synchronously only if you have not already consumed the run, or inspect trace on the same workflow instance if your SDK version attaches trace after stream completion.

Typical post-run trace kinds:

| Event kind | When |
|------------|------|
| `WorkflowStarted` | Run begins |
| `StepStarted` | Step begins (mirrors stream `step_start`) |
| `StepCompleted` | Step ends (mirrors stream `step_complete`) |
| `WorkflowCompleted` | Success |

Stream iterators may surface token deltas before `StepCompleted` appears in trace export.

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| `Expected at least one stream event` | Provider or binding without stream support | Confirm stub path; rebuild SDK |
| TypeScript import error | SDK not built | Run `npm run build` in `sdk-typescript/` |
| Empty token events on stub | Normal for some stub versions | Step events still validate streaming plumbing |
| Expecting HTTP SSE from server | Server SSE not implemented | Use SDK streaming or poll run status |

## Related

| Resource | Link |
|----------|------|
| SDK streaming guide | [sdk-streaming](../guides/streaming/sdk-streaming.md) |
| Browser streaming note | [streaming in the browser](../guides/streaming/streaming-in-the-browser.md) |
| Server streaming gap | [maturity and known gaps](../concepts/maturity-and-known-gaps.md) |
