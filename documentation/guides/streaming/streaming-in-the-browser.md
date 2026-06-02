
# Streaming in the browser

Production static sites use ArcFlow Relay and the static browser SDK. LLM keys stay on the server; the browser never opens a direct provider connection. Today, progressive streaming UX is built by **polling trace events**, not by Server-Sent Events from `arcflow-server`.

True server SSE at `GET /v1/runs/{run_id}/events` is **deferred (streaming deferred)**. Plan integrations accordingly.

## Architecture today

```text
Browser (static SDK)
 → POST /v1/sites/{site_id}/runs (Relay)
 → Relay → POST /v1/runs (arcflow-server)
 ← run_id, trace_id, status

Poll loop:
 → GET /v1/sites/{site_id}/runs/{run_id}/trace (Relay)
 → Relay → GET /v1/runs/{run_id}/trace (server)
 ← TokenEmitted, WorkflowCompleted (metadata only, trace data policy)
```

Relay proxies trace; it does not add prompt text to events.

## Why server streaming matters

| Approach | Status |
|----------|--------|
| SDK `run_stream()` in Node/Python | Implemented |
| Relay trace poll for `TokenEmitted` | Implemented |
| Server SSE `/v1/runs/{id}/events` | **Deferred (streaming deferred)** |

Until server SSE streaming ships, do not depend on an SSE URL in production browser code..

## Polling pattern

After `createRun`, poll trace on an interval until `WorkflowCompleted` or `WorkflowFailed` appears:

```typescript
async function pollTraceForTokens(
 client: ArcFlowSiteClient,
 runId: string,
 onDelta: (completionDelta: number) => void,
): Promise<void> {
 let lastEventCount = 0;
 const intervalMs = 400;
 const maxAttempts = 150;

 for (let attempt = 0; attempt < maxAttempts; attempt++) {
 const trace = await client.getRunTrace(runId);
 const events = trace.events ?? [];

 for (let i = lastEventCount; i < events.length; i++) {
 const ev = events[i];
 if (ev.kind === "TokenEmitted") {
 onDelta(ev.completion_token_delta ?? 0);
 }
 if (ev.kind === "WorkflowCompleted" || ev.kind === "WorkflowFailed") {
 return;
 }
 }
 lastEventCount = events.length;
 await new Promise((r) => setTimeout(r, intervalMs));
 }
 throw new Error("trace poll timeout");
}
```

Adapt method names to your static SDK version (`getRunTrace` or equivalent from `packages/arcflow-static`).

## Building good streaming UX without token text in trace

`TokenEmitted` carries **counts**, not strings:

```json
{
 "kind": "TokenEmitted",
 "run_id": "r1",
 "step_id": "s1",
 "completion_token_delta": 3,
 "prompt_token_delta": 0
}
```

Options for visible streaming text in the browser:

1. **Poll run result** with `GET /v1/sites/{site_id}/runs/{run_id}` and render `result.output` as it grows if your deployment updates partial output (check server behavior for your workflow).
2. **Use token deltas as progress** (spinner, word-count estimate, typing indicator) while final text arrives on completion.
3. **Run chat through a backend** that uses SDK `run_stream()` and forwards deltas over your own WebSocket (keys remain off the browser).

Do not expect trace data policy trace exports to replay exact token strings.

## Recommended poll parameters

| Parameter | Suggested value | Notes |
|-----------|-----------------|-------|
| Interval | 300 to 500 ms | Balance UX vs load |
| Max duration | 60 to 120 s | Align with workflow timeout |
| Backoff on 429 | exponential | If rate limits appear |
| Stop condition | `WorkflowCompleted`, `WorkflowFailed`, or run status terminal | Avoid infinite loops |

Also poll run status (`GET.../runs/{run_id}`) when trace persistence lags behind completion.

## Static SDK run flow

Typical sequence from `packages/arcflow-static`:

1. `client.run(input)` creates the run and may poll until complete for simple chat.
2. For streaming UI, split create and poll: capture `runId`, start trace poll loop, fetch final `RunResult` when status completes.
3. Handle `Interrupted` for HITL or external bindings without treating it as failure.

Site tokens and Origin allowlists are required on Relay. Never embed `ARCFLOW_SERVER_API_KEY` in frontend bundles.

## Security reminders

- Trace poll responses are metadata-only trace; safe to inspect in browser devtools relative to prompt content, but still protect run IDs from unauthorized users.
- Scoped runtime keys limit which workflows a site may start.
- See [Trace data policy](../../concepts/sec-1-and-data-safety.md).

## Related pages

- [SDK streaming](sdk-streaming.md) for in-process `run_stream` / `runStream`
- [Architecture overview](../../concepts/architecture-overview.md) for Relay diagram
- [Maturity and known gaps](../../concepts/maturity-and-known-gaps.md) for server streaming
