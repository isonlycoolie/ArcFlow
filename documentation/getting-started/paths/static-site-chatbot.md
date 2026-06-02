# Static site chatbot


## Before you start

This path walks through the production static-site pattern end to end: server and Relay via Docker, operator provisioning through the admin API, knowledge ingest, chat workflow publish, and a minimal browser client in `examples/static/chat-rag/`.

You should have completed [Server API quickstart](../quickstart-server-api.md) or be comfortable starting the Docker stack. Node.js 18+ is required for the Vite example.

In production, frontend developers do **not** define agents, memory, or ingest documents in browser code. Operators own knowledge and publish records through the ArcFlow Dashboard or admin API. The browser ships two env vars and calls `runPublished()`.

## Concept

The static product splits responsibilities:

| Who | Responsibility |
|-----|----------------|
| Operator / dashboard user | Create site, upload knowledge, set chat instructions, publish workflow |
| Frontend developer | Paste relay URL and site token, wire chat UI, call `runPublished()` |
| ArcFlow Relay | Holds site token, validates Origin, proxies runs to server |
| arcflow-server | Executes published workflow plus RAG against ingested knowledge |

Browser traffic never carries LLM keys or server admin keys. The site token is scoped to Relay routes for that site.

Published workflows resolve by name and semver range. The default chat workflow name is `chat`. The frontend calls `runPublished("chat", "^1.0.0", userMessage)` with no inline agent JSON in the bundle.

Progressive UI today uses **trace polling**, not server SSE. `GET /v1/runs/{run_id}/events` (SSE) is deferred. Plan browser streaming around Relay trace poll and `TokenEmitted` metadata events. See [Streaming in the browser](../../guides/streaming/streaming-in-the-browser.md).

## Prerequisites

| Item | Required |
|------|----------|
| Docker Compose v2 | Server + Relay stack |
| Node.js 18+ | Vite example |
| Bash + curl | Operator scripts |
| Repo clone | Commands assume repo root |

## Step 1: Start server and Relay

From the repository root:

```bash
docker compose -f docker/docker-compose.server.yml up -d --build
```

Confirm readiness:

```bash
curl -sf http://localhost:8080/health
curl -s -o /dev/null -w "%{http_code}\n" http://localhost:8080/ready
```

Wait for **200** on `/ready` before provisioning. Postgres and migrations must be healthy.

## Step 2: Provision site (admin API)

Use the provision script (equivalent to Dashboard → Sites → Create):

```bash
bash scripts/static-provision-site.sh
```

The script prints values to copy:

```text
SITE_ID=s_...
VITE_ARCFLOW_RELAY_URL=http://localhost:8090/v1/sites/s_...
VITE_ARCFLOW_SITE_TOKEN=st_...
```

Export `SITE_ID` for the next steps:

```bash
export SITE_ID=s_... # from script output
```

The script registers `http://localhost:5173` as an allowed origin by default. Override with `ARCFLOW_SITE_ORIGIN` if your dev server uses another port.

Manual equivalent:

```bash
curl -sf -X POST "http://localhost:8080/v1/admin/sites" \
 -H "X-ArcFlow-Admin-Key: dev-admin" \
 -H "Content-Type: application/json" \
 -d '{"display_name":"Static Dev Site","allowed_origins":["http://localhost:5173"]}'
```

When the ArcFlow Dashboard is deployed in your environment, use Sites → Create and copy relay URL and site token from the UI instead.

## Step 3: Ingest knowledge

Sample content lives in `examples/static/chat-rag/kb.txt`. Ingest via script:

```bash
export TEXT_FILE=examples/static/chat-rag/kb.txt
bash scripts/static-ingest-knowledge.sh
```

Manual equivalent:

```bash
curl -sf -X POST "http://localhost:8080/v1/admin/sites/${SITE_ID}/knowledge/ingest" \
 -H "X-ArcFlow-Admin-Key: dev-admin" \
 -H "Content-Type: application/json" \
 -d "{\"text\": \"$(cat examples/static/chat-rag/kb.txt)\", \"key\": \"faq\"}"
```

Do not embed or ingest `kb.txt` from frontend code. Knowledge belongs on the server side only.

## Step 4: Publish chat workflow

```bash
export INSTRUCTIONS="Answer using the knowledge base. Be concise and friendly."
bash scripts/static-publish-chat.sh
```

Manual equivalent:

```bash
curl -sf -X POST "http://localhost:8080/v1/admin/sites/${SITE_ID}/workflows/chat/publish" \
 -H "X-ArcFlow-Admin-Key: dev-admin" \
 -H "Content-Type: application/json" \
 -d '{"instructions": "Answer using the knowledge base. Be concise."}'
```

After publish, semver `^1.0.0` resolves to the latest `chat` workflow for this site.

## Step 5: Configure and run the frontend example

```bash
cd examples/static/chat-rag
```

Create `.env.local` (or export in the shell):

```bash
VITE_ARCFLOW_RELAY_URL=http://localhost:8090/v1/sites/s_... # from provision output
VITE_ARCFLOW_SITE_TOKEN=st_... # from provision output
```

Install and start:

```bash
npm install
npm run dev
```

Open http://localhost:5173 and send a question that appears in `kb.txt`. The browser POSTs to Relay; Relay forwards to the server; the published chat workflow retrieves knowledge and returns a reply.

Production frontend code is intentionally minimal (~30 lines in `src/main.ts`):

```typescript
import { ArcFlowClient, StepForm } from "@arcflow/static";

const client = new ArcFlowClient({
 baseUrl: import.meta.env.VITE_ARCFLOW_RELAY_URL,
 apiKey: import.meta.env.VITE_ARCFLOW_SITE_TOKEN,
 mode: "relay",
});
const form = new StepForm();

const result = await client.runPublished("chat", "^1.0.0", userMessage, {
 initialState: form.toInitialState(),
});
```

See [Static chat widget example](../../examples/static-chat-widget.md) for full UI wiring.

## Step 6: Verify network and trace behavior

Open browser devtools → Network. On send you should see:

| Request | Expected |
|---------|----------|
| `POST.../v1/sites/{site_id}/runs` | **201** with `run_id` |
| Poll until complete | Client receives assistant text |

Optional automated check:

```bash
pytest examples/static/chat-rag/test_static.py -q
```

Operator trace fetch (server direct):

```bash
curl -s "http://localhost:8080/v1/runs/RUN_ID/trace" \
 -H "Authorization: Bearer dev-secret"
```

Trace events you should see for a RAG chat run:

| Event kind | When |
|------------|------|
| `WorkflowStarted` | Published chat run begins |
| `MemoryRetrieved` | Knowledge hit on the question |
| `StepCompleted` | Chat agent step finishes |
| `WorkflowCompleted` | Successful reply |

## Trace polling without server SSE

Server SSE at `GET /v1/runs/{run_id}/events` is **not available** (deferred (server streaming)). Relay does not expose SSE either.

For token-progress UI without waiting for the final `runPublished()` response to block, poll trace through Relay:

```text
Browser
 -> POST /v1/sites/{site_id}/runs (Relay)
 -> GET /v1/sites/{site_id}/runs/{id}/trace (poll loop)
 <- TokenEmitted (counts only, trace data policy)
 <- WorkflowCompleted
```

`TokenEmitted` carries token **counts**, not prompt or completion strings. Build UX around progress indicators or reveal the final `result.output` when polling completes. Full pattern: [Streaming in the browser](../../guides/streaming/streaming-in-the-browser.md).

Do not depend on an SSE URL in production browser code until server SSE streaming ships.

## Step 7: Verify origin enforcement

Remove `http://localhost:5173` from the site allowed origins (admin API or dashboard), then retry chat from the browser. Relay should reject the request before upstream execution.

Re-add the origin and confirm chat works again.

| Check | Expected |
|-------|----------|
| Allowed origin chat | Success |
| Disallowed origin | Blocked at Relay |
| Production bundle | Site token only, no LLM or server runtime keys |
| Workflow in bundle | None; `runPublished("chat", "^1.0.0",...)` only |

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| `/ready` returns 503 | Postgres or migrations not finished | Wait and retry; check `docker compose logs arcflow-migrate` |
| CORS error in browser | Origin not in site allowed list | Update site `allowed_origins` via admin API or dashboard |
| 401 on Relay POST | Invalid or expired site token | Re-run `static-provision-site.sh` or copy token from dashboard |
| 403 / origin rejected | Request from unlisted origin | Add production and dev origins to site config |
| Empty or generic answers | Knowledge not ingested | Re-run `static-ingest-knowledge.sh` with correct `SITE_ID` |
| `runPublished` workflow not found | Chat not published | Re-run `static-publish-chat.sh` |
| Inline workflow rejected | `allow_inline: false` (production default) | Use publish flow; do not ship `main-dev-direct.ts` |
| Chat hangs forever | Server down or run stuck | Poll `GET /v1/runs/{id}` on server; check logs |
| Expected SSE stream | Server SSE not shipped | Use trace poll; see [Streaming in the browser](../../guides/streaming/streaming-in-the-browser.md) |
| Keys in frontend bundle | Used direct mode by mistake | Use Relay mode only in production; see `src/main-dev-direct.ts` for local engine debug only |

## Advanced: direct mode (internal dev only)

The [Static chat widget example](../../examples/static-chat-widget.md) documents a dev-only direct-runtime path. Do not ship that pattern to production. Keys and workflow shape belong in dashboard/Relay, not the static bundle.

## Verify (summary)

| Check | Expected |
|-------|----------|
| Stack up | `/ready` 200 |
| Site provisioned | Relay URL and token printed |
| Knowledge ingested | Script exits 0 |
| Chat published | Script exits 0 |
| Local chat UI | Answer grounded in `kb.txt` content |
| Origin test | Disallowed origin blocked |
| No SSE dependency | Trace poll documented for streaming UX |

## Next

| Goal | Document |
|------|----------|
| Track F tutorial | [Track F: Static product](../../tutorials/track-f-static-product.md) |
| Browser SDK API | [Browser SDK API](../../static-product/browser-sdk-api.md) |
| Relay request path | [Relay request path](../../relay/request-path.md) |
| Security model | [Static product security model](../../static-product/security-model.md) |
| BYO Relay deployment | [Relay BYO deployment](../../examples/relay-byo-deployment.md) |
| Integrating server features | [Integrating track](../integrating/README.md) |
