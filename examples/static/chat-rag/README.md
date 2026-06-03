# Landing-Page Support Chat (Production)

Production static-site chatbot for a marketing or support landing page. Visitors ask questions; answers come from knowledge you uploaded in the **ArcFlow Dashboard**, not from code in this repo.

## Who does what

| Role | Work |
|------|------|
| **Site owner / dashboard user** | Upload docs, write chat instructions, publish workflow |
| **Frontend developer** | Wire UI + two env vars + `runPublished()` |

The frontend file [`src/main.ts`](src/main.ts) is intentionally minimal (~30 lines). No agents, no memory config, no ingest logic.

## Dashboard setup (before frontend work)

1. **Create site** (Sites, Create): copy relay URL and site token once
2. **Upload knowledge** (Knowledge tab): add your FAQs and docs  
   See [RAG document upload guide](../../ArcFlow_Improvement_Plans/arcflow-static-product-vision/10-rag-document-upload-guide.md) for how to structure uploads for good chunking
3. **Configure chat** (Chat tab): set instructions (e.g. “You are Acme Corp support…”), then **Save & publish**

After publish, the published workflow name is `chat` (default). The frontend calls that by name, no workflow definition in browser code.

## Frontend env (production)

```bash
VITE_ARCFLOW_RELAY_URL=https://relay.arcflow.app/v1/sites/s_abc123
VITE_ARCFLOW_SITE_TOKEN=st_live_xxxxxxxx
```

Add your production origin under Dashboard, Sites, Allowed origins (e.g. `https://www.yoursite.com` and `http://localhost:5173` for local dev).

## Run locally

```bash
cd examples/static/chat-rag
npm install
npm run dev
```

Open http://localhost:5173, chat hits Relay with your site token.

## Frontend code (all you ship)

```typescript
import { ArcFlowClient, StepForm } from "@arcflow/static";

const client = new ArcFlowClient({
  baseUrl: import.meta.env.VITE_ARCFLOW_RELAY_URL,
  apiKey: import.meta.env.VITE_ARCFLOW_SITE_TOKEN,
  mode: "relay",
});

await client.runPublished("chat", "^1.0.0", userMessage, {
  initialState: form.toInitialState(), // optional multi-turn
});
```

See [`src/main.ts`](src/main.ts) for the full UI wiring.

## Local dev without dashboard UI

Use the admin API until the dashboard ships:

```bash
docker compose -f docker/docker-compose.server.yml up -d
bash scripts/relay-provision-site.sh

# Ingest sample knowledge (dashboard will replace this UX)
curl -X POST "http://localhost:8080/v1/admin/sites/{site_id}/knowledge/ingest" \
  -H "X-ArcFlow-Admin-Key: dev-admin" \
  -H "Content-Type: application/json" \
  -d '{"text": "'"$(cat kb.txt)"'", "key": "faq"}'

# Publish chat workflow
curl -X POST "http://localhost:8080/v1/admin/sites/{site_id}/workflows/chat/publish" \
  -H "X-ArcFlow-Admin-Key: dev-admin" \
  -H "Content-Type: application/json" \
  -d '{"instructions": "Answer using the knowledge base. Be concise."}'
```

[`kb.txt`](kb.txt) is sample content for dashboard upload testing, **do not** embed or ingest it from frontend code.

## Advanced: direct mode (local development only)

[`src/main-dev-direct.ts`](src/main-dev-direct.ts) defines inline `Agent` + `MemoryConfig` for local engine debugging with CORS. Not for production, keys and workflow shape belong in dashboard/Relay, not the static bundle.

## Tests

```bash
pytest examples/static/chat-rag/test_static.py -q
```
