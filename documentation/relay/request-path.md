**Audience:** `[frontend]`

# Relay request path

This page walks through a single chat message from the browser to the ArcFlow engine and back. Sequence assumes `mode: "relay"` in `@arcflow/static` and a site provisioned via admin API.

## Sequence

```text
  Browser                Relay                  Server              Engine
     |                      |                      |                   |
     | POST /v1/sites/s/runs|                      |                   |
     | Bearer site_token    |                      |                   |
     | Origin: https://app  |                      |                   |
     |--------------------->|                      |                   |
     |                      | validate token       |                   |
     |                      | check Origin         |                   |
     |                      | check rate limit     |                   |
     |                      | POST /v1/runs        |                   |
     |                      | Bearer scoped key    |                   |
     |                      |--------------------->|                   |
     |                      |                      | auth + validate   |
     |                      |                      |------------------>|
     |                      |                      | execute workflow  |
     |                      |                      |<------------------|
     |                      | 201 run_id, status   |                   |
     |                      |<---------------------|                   |
     | 201 run_id, status   |                      |                   |
     |<---------------------|                      |                   |
     |                      |                      |                   |
     | GET .../runs/{id}    |                      |                   |
     | (poll loop)          | GET /v1/runs/{id}    |                   |
     |--------------------->|--------------------->|                   |
     | status: Completed    |                      |                   |
     |<---------------------|<---------------------|                   |
     |                      |                      |                   |
     | GET .../trace        | GET .../trace        |                   |
     |--------------------->|--------------------->|                   |
     | TokenEmitted (meta)  |                      |                   |
     |<---------------------|<---------------------|                   |
```

## Step 1: Browser create run

The static SDK posts to Relay with the site token (not the server master key):

```typescript
const client = new ArcFlowClient({
  baseUrl: import.meta.env.VITE_ARCFLOW_RELAY_URL,
  apiKey: import.meta.env.VITE_ARCFLOW_SITE_TOKEN,
  mode: "relay",
  siteId: import.meta.env.VITE_ARCFLOW_SITE_ID,
});

const result = await client.runPublished("chat", "^1.0.0", userMessage);
```

HTTP equivalent:

```bash
curl -s -X POST "http://localhost:8090/v1/sites/s_dev/runs" \
  -H "Authorization: Bearer st_live_devtoken" \
  -H "Origin: http://localhost:5173" \
  -H "Content-Type: application/json" \
  -d '{"workflow_ref":{"name":"chat","version":"^1.0.0"},"input":"Hello"}'
```

Payload uses `workflow_ref` so agent definitions stay on the server registry, not in the JS bundle.

## Step 2: Relay middleware

1. **Site lookup** from in-memory store (loaded from Postgres-backed admin or `ARCFLOW_RELAY_SITES_JSON`).
2. **Token verify** against configured site token hash.
3. **Origin check** against `allowed_origins` (see [origin-and-rate-limiting.md](origin-and-rate-limiting.md)).
4. **Rate limit** token bucket per site (`429` when exceeded).

Implementation: `server/arcflow-relay/src/middleware/site_auth.rs`.

## Step 3: Upstream proxy

Relay forwards to `ARCFLOW_UPSTREAM_URL` (default `http://arcflow-server:8080`):

- Path rewritten from `/v1/sites/{id}/runs` to `/v1/runs`
- Auth header replaced with site's `upstream_runtime_key` (must exist in `ARCFLOW_STATIC_RUNTIME_KEYS` or match master key policy)
- `Idempotency-Key` forwarded if present

Implementation: `server/arcflow-relay/src/handlers/proxy.rs`.

## Step 4: Poll status

The static SDK polls until terminal status. Each poll hits Relay, which proxies `GET /v1/runs/{run_id}`.

```bash
curl -s "http://localhost:8090/v1/sites/s_dev/runs/RUN_ID" \
  -H "Authorization: Bearer st_live_devtoken" \
  -H "Origin: http://localhost:5173"
```

## Step 5: Trace for streaming UX

For progressive UI, poll trace and read `TokenEmitted` events (byte/token counts only, SEC-1). No server SSE until FP-2.

```bash
curl -s "http://localhost:8090/v1/sites/s_dev/runs/RUN_ID/trace" \
  -H "Authorization: Bearer st_live_devtoken" \
  -H "Origin: http://localhost:5173"
```

See [guides/streaming/streaming-in-the-browser.md](../guides/streaming/streaming-in-the-browser.md).

## HITL through Relay

When a run returns `Interrupted`, the static SDK throws `WorkflowInterruptedError` with `approvalKey`. Approval may go through server API from a trusted backend; Relay approve proxy depends on scoped key configuration.

## Related pages

- [relay/overview.md](overview.md)
- [static-product/browser-sdk-api.md](../static-product/browser-sdk-api.md)

**Source:** capabilities reference §14.1, §2; `server/arcflow-relay/src/handlers/proxy.rs`, `server/arcflow-relay/src/middleware/site_auth.rs`; Appendix B (Relay routes).
