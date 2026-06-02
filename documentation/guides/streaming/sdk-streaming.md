
# SDK streaming

Python and TypeScript SDKs expose token and step events through streaming iterators while the Rust engine runs in-process. Traces record `StreamChunkReceived` and `TokenEmitted` with byte and token **counts only** (trace data policy). Server-side SSE at `GET /v1/runs/{id}/events` is **not implemented (streaming deferred)**; use SDK streaming for in-process UX or poll trace from HTTP clients.

## Enable streaming in exec_config

```json
{
 "stream": { "enabled": true }
}
```

When disabled, the SDK returns the full step output at completion with no incremental token events.

Python passes exec config via run options or workflow defaults depending on your wrapper; TypeScript accepts `{ stream: { enabled: true } }` on `runStream`.

## Python: run_stream

`run_stream` is async; iterate with `async for`:

```python
import asyncio
from arcflow import Agent, Workflow


async def main() -> None:
 wf = Workflow("chat_stream_demo")
 wf.step(Agent(name="assistant", role="helper", instructions="Reply briefly."))

 async for event in wf.run_stream("Hello from ArcFlow"):
 if event.type == "token":
 print(event.text, end="", flush=True)
 elif event.type == "step_start":
 print(f"\n[step start: {event.step_id}]")
 elif event.type == "step_complete":
 print(f"[step complete: {event.step_id} in {event.duration_ms}ms]")


asyncio.run(main())
```

See `examples/streaming/chat_stream.py`. Stub agents work without LLM API keys.

### Stream event types (Python)

| event.type | Fields | Use |
|------------|--------|-----|
| `token` | `text` (delta at SDK layer) | Incremental UI rendering |
| `step_start` | `step_id` | Progress indicator |
| `step_complete` | `step_id`, `duration_ms` | Step boundary |
| other | varies | Diagnostics |

SDK token text is available to your application code during the run. Persisted traces and HTTP trace exports do **not** store raw token strings.

## TypeScript: runStream

```typescript
import { Agent, Workflow } from "@arcflow/sdk";

async function main(): Promise<void> {
 const wf = new Workflow({ name: "chat_stream_demo" });
 wf.step(
 new Agent({
 name: "assistant",
 role: "helper",
 instructions: "Reply briefly.",
 }),
 );

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
}

main();
```

See `examples/streaming/chat_stream.ts`.

## Trace events during streaming

After the run, inspect metadata-only trace events:

```json
[
 { "kind": "StreamChunkReceived", "run_id": "r1", "step_id": "s1", "chunk_bytes": 64 },
 { "kind": "TokenEmitted", "run_id": "r1", "step_id": "s1", "completion_token_delta": 4, "prompt_token_delta": 0 },
 { "kind": "StreamChunkReceived", "run_id": "r1", "step_id": "s1", "chunk_bytes": 32 },
 { "kind": "TokenEmitted", "run_id": "r1", "step_id": "s1", "completion_token_delta": 2, "prompt_token_delta": 0 }
]
```

Use these events for dashboards and latency analysis, not for reconstructing full completions from trace storage alone.

## Blocking run vs stream

| API | When to use |
|-----|-------------|
| `workflow.run()` | Batch jobs, tests, simple scripts |
| `workflow.run_stream()` / `runStream` | Chat UIs, progressive rendering in Node or Python services |

Both paths share the same engine and trace data policy trace rules.

## server SSE deferred

Remote clients cannot open an SSE channel to `arcflow-server` today. HTTP integrations should poll `GET /v1/runs/{id}` or `GET /v1/runs/{id}/trace`. Browser apps use Relay trace polling; see [streaming in the browser](streaming-in-the-browser.md).

## Related pages

- [Streaming in the browser](streaming-in-the-browser.md) for Relay polling pattern
- [Execution traces](../observability/execution-traces.md) for reading traces after a stream completes
- [Maturity and known gaps](../../concepts/maturity-and-known-gaps.md) for server streaming status
