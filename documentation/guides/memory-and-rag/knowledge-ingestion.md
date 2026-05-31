
# Knowledge ingestion

Knowledge ingestion writes text into a vector namespace so agents with `memory_type: "vector"` can retrieve it at run time. Ingestion chunks source text, embeds chunks, and upserts into Qdrant. Operators using the static product ingest via the admin API; developers can ingest through SDK examples or direct pipeline scripts.

Retrieval behavior: [Vector RAG pipeline](vector-rag-pipeline.md). Memory model: [Memory types](memory-types.md).

## Ingest pipeline

```text
Source text → Chunk (size, overlap) → Embed → Upsert Qdrant collection (namespace)
```

Chunking config on the agent or ingest request:

```json
{
  "chunking": {
    "chunk_size": 512,
    "chunk_overlap": 64
  }
}
```

Embedding string example: `openai/text-embedding-3-small` with `OPENAI_API_KEY` and `ARCFLOW_EMBEDDING_PROVIDER` set on the server.

## Admin API (static product)

Bearer `ARCFLOW_ADMIN_API_KEY`. After site creation (`POST /v1/admin/sites`), each site receives a `kb_namespace` used for knowledge storage.

### Ingest text

```http
POST /v1/admin/sites/{site_id}/knowledge/ingest
Authorization: Bearer <ARCFLOW_ADMIN_API_KEY>
Content-Type: application/json
```

Body:

```json
{
  "text": "Refunds are processed within 5 business days when the return is received.",
  "key": "refund-policy"
}
```

| Field | Role |
|-------|------|
| `text` | Raw content to chunk and embed |
| `key` | Logical document id for updates and eviction |

Repeat for each FAQ section or document. Keys help replace content without duplicating chunks.

### Site creation context

```json
POST /v1/admin/sites

{
  "display_name": "Acme Support",
  "allowed_origins": ["https://www.acme.com"],
  "rate_limit_rpm": 60,
  "allow_inline": false,
  "default_workflow_name": "chat",
  "chat_instructions": "Answer from knowledge only."
}
```

Response includes `kb_namespace` (use in published agent memory config):

```json
{
  "site_id": "site-uuid",
  "relay_url": "https://relay.example.com",
  "site_token": "shown-once-token",
  "kb_namespace": "site-acme-kb-uuid"
}
```

### Publish chat workflow

After ingest, publish so browser clients resolve the workflow:

```json
POST /v1/admin/sites/{site_id}/workflows/chat/publish

{
  "instructions": "Answer only from ingested knowledge. Say when unsure.",
  "version": "1.0.1"
}
```

Published agents should reference the site namespace:

```json
{
  "memory_config": {
    "memory_type": "vector",
    "scope": "workflow",
    "namespace": "site-acme-kb-uuid",
    "embedding": "openai/text-embedding-3-small",
    "retrieval": { "mode": "hybrid", "top_k": 5 }
  }
}
```

Browser: `runPublished("chat", "^1.0.0", userMessage)` via Relay. See [Workflow registry](../workflows/workflow-registry.md) and [Server API quickstart](../../getting-started/quickstart-server-api.md).

## Developer ingest (examples)

Example directories:

| Path | Pattern |
|------|---------|
| `examples/rag/` | SDK vector store ingest |
| `examples/static/chat-rag/` | Admin + Relay + static SDK end-to-end |
| `examples/online-application-chatbot/` | Full static product |

Typical SDK pattern (conceptual):

```python
from arcflow.memory import VectorStore

store = VectorStore(
    namespace="product-docs",
    embedding="openai/text-embedding-3-small",
    chunk_size=512,
    chunk_overlap=64,
)

store.ingest_text(key="getting-started", text=open("docs/intro.md").read())
```

Exact API names follow `sdk-python` examples; align namespace with agent `memory_config.namespace`.

## Prerequisites

| Component | Variable / service |
|-----------|------------------|
| Qdrant | `ARCFLOW_QDRANT_URL` |
| Embeddings | `ARCFLOW_EMBEDDING_PROVIDER`, provider API key |
| Server (admin path) | `ARCFLOW_POSTGRESQL_URL`, `ARCFLOW_ADMIN_API_KEY` |
| Hybrid search | `ARCFLOW_QDRANT_HYBRID=true` optional |

Production: do not use stub embedding provider. See [Hybrid retrieval and reranking](hybrid-retrieval-and-reranking.md).

## Verify ingest

1. Run a workflow with vector agent pointing at the namespace
2. Query with text that appears only in ingested docs
3. Trace shows `MemoryRetrieved` with `chunk_count > 0`:

```json
{
  "kind": "MemoryRetrieved",
  "run_id": "r1",
  "agent_name": "support",
  "chunk_count": 3,
  "total_bytes": 2400
}
```

4. Answer should reflect ingested facts (manual review; traces do not contain chunk text per SEC-1)

## Operational scripts

Reference CI smoke implementations:

- `scripts/static-provision.sh`
- `scripts/static-smoke.sh`

## Updating and deleting content

Re-ingest with the same `key` to replace logical documents per admin API semantics. Monitor `MemoryEvicted` traces if TTL or capacity policies apply on persistent stores (vector collections follow Qdrant collection management).

## Related pages

- [Vector RAG pipeline](vector-rag-pipeline.md)
- [Memory types](memory-types.md)
- [Surfaces and when to use them](../../concepts/surfaces-and-when-to-use-them.md)
- [Architecture overview](../../concepts/architecture-overview.md)
