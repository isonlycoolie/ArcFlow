**Audience:** `[frontend]`

# Streaming in the browser

Production static sites use ArcFlow Relay and the static browser SDK. LLM keys stay on the server; the browser never opens a direct provider connection. Today, progressive streaming UX is built by **polling trace events**, not by Server-Sent Events from `arcflow-server`.

True server SSE at `GET /v1/runs/{run_id}/events` is **deferred (FP-2)**. Plan integrations accordingly.

## Architecture today

```text
Browser (static SDK)
  → POST /v1/sites/{site_id}/runs        (Relay)
  → Relay → POST /v1/runs                (arcflow-server)
  ← run_id, trace_id, status

Poll loop:
  → GET /v1/sites/{site_id}/runs/{run_id}/trace   (Relay)
  → Relay → GET /v1/runs/{run_id}/trace           (server)
  ← TokenEmitted, WorkflowCompleted (metadata only, SEC-1)
```

Relay proxies trace; it does not add prompt text to events.

## Why FP-2 matters

| Approach | Status |
|----------|--------|
| SDK `run_stream()` in Node/Python | Implemented |
| Relay trace poll for `TokenEmitted` | Implemented |
| Server SSE `/v1/runs/{id}/events` | **Deferred (FP-2)** |

Until FP-2 ships, do not depend on an SSE URL in production browser code. Feature plan: `feat/fp-2-server-streaming`.

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

Do not expect SEC-1 trace exports to replay exact token strings.

## Recommended poll parameters

| Parameter | Suggested value | Notes |
