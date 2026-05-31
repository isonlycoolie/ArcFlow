# Static Chat RAG Example

Minimal static-site chat using `@arcflow/static` with inline workflow + vector memory config.

## Prerequisites

```bash
docker run -d -p 6333:6333 qdrant/qdrant:v1.12.5
export ARCFLOW_SERVER_API_KEY=dev-secret
export ARCFLOW_QDRANT_URL=http://localhost:6333
export ARCFLOW_CORS_ORIGINS=http://localhost:5173
export OPENAI_API_KEY=sk-...
# Start arcflow-server with Postgres (see server/README.md)
```

Ingest knowledge (server-side once):

```python
from arcflow.memory import VectorStore
VectorStore().ingest("support-kb", "kb", open("kb.txt").read())
```

## Run demo UI

```bash
cd examples/static/chat-rag
npm install
npm run dev
```

## SDK usage

See `src/main.ts` — defines `Agent` + `MemoryConfig` in browser, calls `ArcFlowClient.runWorkflow()` in `direct` mode.
