**Audience:** `[developer]` `[operator]`

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
