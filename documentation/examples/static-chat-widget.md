# Static chat widget example

**Audience:** `[frontend]` `[operator]`

This walkthrough provisions a static-site support chat using ArcFlow Relay and published workflows. The browser bundle calls `runPublished()` only; agents, memory, and knowledge live in the dashboard or admin API.

Primary example: [`examples/static/chat-rag/`](../../examples/static/chat-rag/). Index: [`examples/static/README.md`](../../examples/static/README.md).

## What this example demonstrates

Production static sites never embed LLM keys or workflow definitions in client JavaScript. The operator uploads knowledge, publishes a `chat` workflow, and the frontend developer wires two env vars plus a thin UI. Relay validates Origin and proxies runs to `arcflow-server`.

## Prerequisites

| Item | Required |
|------|----------|
| Server + Relay stack | `docker compose -f docker/docker-compose.server.yml up -d` |
| Site provisioned | Dashboard or `scripts/static-provision-site.sh` |
| Node.js | 18+ for Vite dev server |
| Allowed origins | Include `http://localhost:5173` and production domain |
| Tutorial track | [Track F](../tutorials/track-f-static-product.md) |

## Step 1: Operator setup (dashboard or scripts)

From repository root:

```bash
docker compose -f docker/docker-compose.server.yml up -d
bash scripts/static-provision-site.sh
export SITE_ID=...  # from script output
export TEXT_FILE=examples/static/chat-rag/kb.txt
bash scripts/static-ingest-knowledge.sh
bash scripts/static-publish-chat.sh
```

Copy relay URL and site token for frontend env vars.

## Step 2: Configure frontend env

In `examples/static/chat-rag/`:

```bash
VITE_ARCFLOW_RELAY_URL=http://localhost:8090/v1/sites/s_dev
VITE_ARCFLOW_SITE_TOKEN=st_live_devtoken
```

Use values from your site record, not placeholders, in real deployments.

## Step 3: Run the chat UI locally

```bash
cd examples/static/chat-rag
npm install
npm run dev
```

Open http://localhost:5173 and send a message. Traffic goes to Relay with Bearer site token and Origin header.

Production client pattern from [`src/main.ts`](../../examples/static/chat-rag/src/main.ts):

```typescript
import { ArcFlowClient } from "@arcflow/static";

const client = new ArcFlowClient({
  baseUrl: import.meta.env.VITE_ARCFLOW_RELAY_URL,
  apiKey: import.meta.env.VITE_ARCFLOW_SITE_TOKEN,
  mode: "relay",
});

await client.runPublished("chat", "^1.0.0", userMessage);
```

## Step 4: Verify origin enforcement

Request from a disallowed origin should fail at Relay (403 or CORS rejection). Add your test origin in Sites allowed origins, retry, and confirm success.

Optional automated check:

```bash
pytest examples/static/chat-rag/test_static.py -q
```

## Expected output

Browser chat returns assistant replies grounded on ingested knowledge (stub or real provider depending on server config). Network tab shows POST to Relay `/v1/sites/{id}/runs` with 201 response and subsequent poll or stream per client settings.

Pass criteria:

| Check | Expected |
|-------|----------|
| Chat message succeeds from allowed origin | 201 from Relay |
| Disallowed origin blocked | Error before server execution |
| No secrets in bundle | Only site token env at build time |

## Trace events you should see

Fetch trace via Relay or server (operator tools):

| Event kind | When |
|------------|------|
| `WorkflowStarted` | Published chat workflow run |
| `MemoryRetrieved` | When knowledge base matches query |
| `StepCompleted` | Chat agent step |
| `WorkflowCompleted` | Successful reply |

Traces are metadata only (SEC-1). End users do not receive raw trace JSON in the widget unless you add operator tooling.

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| CORS or 403 from Relay | Origin not allowlisted | Add exact scheme + host in site settings |
| Empty answers | Knowledge not ingested | Re-run ingest script or dashboard upload |
| 401 on Relay | Wrong site token | Regenerate token; update env |
| Agents defined in browser | Wrong integration path | Remove inline Agent code; use publish flow only |

Internal dev only: [`src/main-dev-direct.ts`](../../examples/static/chat-rag/src/main-dev-direct.ts) bypasses Relay for engine debugging. Do not ship to production.

## Related

| Resource | Link |
|----------|------|
| Relay BYO | [relay-byo-deployment.md](relay-byo-deployment.md) |
| Multi-turn intake bot | [`online-application-chatbot`](../../examples/static/online-application-chatbot/README.md) |
| Tutorial track | [Track F](../tutorials/track-f-static-product.md) |

**Source:** [`examples/static/chat-rag/`](../../examples/static/chat-rag/), [`examples/static/README.md`](../../examples/static/README.md); capabilities reference §25, §28 Track F; [knowledge ingestion](../guides/memory-and-rag/knowledge-ingestion.md).
