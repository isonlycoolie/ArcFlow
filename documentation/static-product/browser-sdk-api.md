**Audience:** `[frontend]`

# Browser SDK API (`@arcflow/static`)

Package path: `packages/arcflow-static`. Production browser client for published workflows via Relay, direct server (dev only), or BFF.

## Installation

```bash
npm install @arcflow/static
```

Build from monorepo:

```bash
cd packages/arcflow-static && npm run build
```

## ArcFlowClient

Primary entry point.

```typescript
import { ArcFlowClient } from "@arcflow/static";

const client = new ArcFlowClient({
  baseUrl: import.meta.env.VITE_ARCFLOW_RELAY_URL,
  apiKey: import.meta.env.VITE_ARCFLOW_SITE_TOKEN,
  mode: "relay",
  siteId: import.meta.env.VITE_ARCFLOW_SITE_ID,
});
```

### Constructor options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `baseUrl` | string | required | Relay host or server URL (no trailing slash) |
| `apiKey` | string | required | Site token (relay) or server key (direct) |
| `mode` | `"relay"` \| `"direct"` \| `"bff"` | `"relay"` | Routing mode |
| `siteId` | string | parsed from URL | Required for relay if not in baseUrl path |
| `useArcFlowHeader` | boolean | `true` for direct | Use `X-ArcFlow-Api-Key` vs Bearer |

### runPublished

Resolve semver range from registry and execute:

```typescript
const result = await client.runPublished(
  "chat",
  "^1.0.0",
  userMessage,
  { initialState: { locale: "en" } },
);
```

Sends:

```json
{
  "workflow_ref": { "name": "chat", "version": "^1.0.0" },
  "input": "user message",
  "exec_config": { "initial_state": { "locale": "en" } }
}
```

Polls until `Completed` or throws on `Failed` / `Interrupted`.

### runWorkflow

Inline workflow (dev or `allow_inline: true` sites only):

```typescript
import { Workflow, Agent } from "@arcflow/static";

const wf = new Workflow({ name: "demo", runtimeUrl: serverUrl });
wf.addStep({ agent: new Agent({ name: "a", role: "r", instructions: "..." }) });
const result = await client.runWorkflow(wf, "hello");
```

Production sites should set `allow_inline: false` so browsers cannot override published definitions.

### getRun / pollUntilComplete

```typescript
const detail = await client.getRun(runId);
const final = await client.pollUntilComplete(runId, 500, 60);
```

### Registry helpers

```typescript
await client.publishWorkflow(workflow, "1.0.0");
await client.resolveWorkflow("chat", "^1.0.0");
```
