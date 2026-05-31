# Static Site Production Examples

Real-world patterns for shipping ArcFlow on static sites (Vite, Next.js static export, Netlify, Vercel, S3).

**Production path:** ArcFlow Dashboard + ArcFlow Relay. The frontend developer does **not** define agents, memory, or ingest documents in browser code.

| Who | Responsibility |
|-----|----------------|
| **Dashboard user** (you or your customer) | Create site, upload knowledge, set chat instructions, publish workflow |
| **Frontend developer** | Paste two env vars, wire a chat UI, call `runPublished()` |
| **ArcFlow Relay** | Holds site token, validates Origin, proxies runs |
| **Arcflow-server** | Executes published workflow + RAG against dashboard-ingested knowledge |

## Examples

| Example | Use case | Frontend code |
|---------|----------|---------------|
| [`chat-rag/`](chat-rag/) | Landing-page support chat with RAG | ~30 lines, relay + `runPublished()` only |
| [`online-application-chatbot/`](online-application-chatbot/) | Multi-turn intake + external callback | Relay + published workflow ref |

## What frontend developers never do in production

- Define `Agent`, `MemoryConfig`, or `Workflow` in the browser bundle
- Run Python ingest scripts or call vector APIs
- Store LLM or server API keys in the repo
- Chunk or embed documents client-side

All of that lives in the **ArcFlow Dashboard** (Knowledge tab, Chat tab, Publish).

## Setup flow (every production site)

1. **Dashboard → Sites → Create site**, copy `VITE_ARCFLOW_RELAY_URL` and `VITE_ARCFLOW_SITE_TOKEN`
2. **Dashboard → Knowledge**, upload FAQs, docs, policies ([upload guide](../../ArcFlow_Improvement_Plans/arcflow-static-product-vision/10-rag-document-upload-guide.md))
3. **Dashboard → Chat**, set instructions, click **Save & publish**
4. **Frontend**, add env vars, implement chat UI calling `runPublished("chat", "^1.0.0", message)`
5. **Deploy** to CDN, no server-side code required

## Local development without dashboard UI

Until the dashboard ships, use the admin API + provision script:

```bash
bash scripts/relay-provision-site.sh
# Then ingest + publish via admin API (see chat-rag/README.md)
```

## Advanced: direct mode (internal dev only)

For local debugging with CORS and scoped runtime keys, see `chat-rag/src/main-dev-direct.ts`. **Do not ship this to production.**
