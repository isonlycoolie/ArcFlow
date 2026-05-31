**Audience:** `[developer]`

# Vector RAG pipeline

Vector memory connects agent turns to a Qdrant collection through embed, store, retrieve, and optional rerank stages. The engine injects retrieved context into the agent prompt while traces record only chunk counts and byte totals (SEC-1).

Types overview: [Memory types](memory-types.md). Retrieval tuning: [Hybrid retrieval and reranking](hybrid-retrieval-and-reranking.md). Ingest paths: [Knowledge ingestion](knowledge-ingestion.md).

## Pipeline stages

```text
Ingest → Chunk → Embed → Upsert (Qdrant)
                ↓
Run turn → Query embed → Retrieve (dense / hybrid) → Rerank? → Prompt injection
                ↓
         MemoryRetrieved trace (chunk_count, total_bytes)
```

## Agent memory_config

```json
{
  "memory_config": {
    "memory_type": "vector",
    "scope": "workflow",
    "namespace": "product-docs",
    "embedding": "openai/text-embedding-3-small",
    "retrieval": {
      "mode": "hybrid",
      "top_k": 8,
      "dense_weight": 0.65,
      "sparse_weight": 0.35,
      "rerank": {
        "provider": "cohere",
        "model": "rerank-english-v3.0",
        "top_n": 4
      }
    },
    "chunking": {
      "chunk_size": 512,
      "chunk_overlap": 64
    }
  }
}
```

| Stage | Config field |
|-------|--------------|
| Collection identity | `namespace` |
| Embedding model | `embedding` provider string |
| Retrieval | `retrieval.mode`, `top_k`, weights |
| Rerank | `retrieval.rerank` (optional) |
| Ingest chunking | `chunking.chunk_size`, `chunk_overlap` |

## Stage 1: Ingest

Text is split using `MemoryChunkingConfig`:

```json
{
  "chunking": {
    "chunk_size": 512,
    "chunk_overlap": 64
  }
}
```

Each chunk is embedded via the `embedding` string (for example `openai/text-embedding-3-small`) and upserted into the Qdrant collection named by `namespace`.

SDK and admin ingest paths differ; see [Knowledge ingestion](knowledge-ingestion.md).

## Stage 2: Retrieve

On each agent turn, the runtime embeds the query (typically derived from the current prompt context) and searches Qdrant.

| mode | Behavior |
|------|----------|
| `dense` | Vector similarity only |
| `hybrid` | Dense + sparse when `ARCFLOW_QDRANT_HYBRID=true` |

Set env:

```bash
export ARCFLOW_QDRANT_URL=http://localhost:6333
export ARCFLOW_QDRANT_HYBRID=true
export ARCFLOW_EMBEDDING_PROVIDER=openai
export OPENAI_API_KEY=sk-...
```

## Stage 3: Rerank

Optional Cohere rerank narrows `top_k` chunks to `top_n` before prompt injection. Requires `COHERE_API_KEY`.

```json
{
  "rerank": {
    "provider": "cohere",
    "model": "rerank-english-v3.0",
    "top_n": 3
  }
}
```

Failure maps to `RerankError` (502). Engine may degrade per configuration; watch for `MemoryDegraded` traces.

## Stage 4: Trace (SEC-1)

```json
{
  "kind": "MemoryRetrieved",
  "run_id": "r1",
  "step_id": "s1",
  "agent_name": "support",
  "chunk_count": 5,
  "total_bytes": 4200
}
```

No chunk text in traces or persisted `arcflow_trace_events`. [SEC-1 and data safety](../../concepts/sec-1-and-data-safety.md).

## Full agent example

```json
{
  "id": "00000000-0000-4000-8000-000000000020",
  "name": "support",
  "role": "Support agent",
  "instructions": "Answer only from retrieved knowledge. Say when unsure.",
  "memory_config": {
    "memory_type": "vector",
    "scope": "workflow",
    "namespace": "acme-support",
    "embedding": "openai/text-embedding-3-small",
    "retrieval": { "mode": "hybrid", "top_k": 5 },
    "chunking": { "chunk_size": 512, "chunk_overlap": 64 }
  },
  "provider": {
    "provider_id": "openai",
    "model": "gpt-4o-mini",
    "api_key_env": "OPENAI_API_KEY"
  }
}
```

## Docker stack

Local RAG development typically uses `docker/docker-compose.server.yml` (Postgres + Qdrant + server). See [Server API quickstart](../../getting-started/quickstart-server-api.md).

## Verification

After ingest and run:

1. `result.status` is `Completed`
2. Trace contains `MemoryRetrieved` with `chunk_count > 0`
3. No `MemoryDegraded` unless testing failure paths

Examples: `examples/rag/`, `examples/static/chat-rag/`, `examples/online-application-chatbot/`.

## Production checklist

| Requirement | Reason |
|-------------|--------|
| Real Qdrant URL | Stub vector backend is dev-only |
| Non-stub embedding provider | Deterministic stub embeddings do not generalize |
| Namespace per tenant or site | Isolation for static product KB |
| Cohere key if rerank enabled | Avoid silent rerank failures |

## Related pages

- [Memory types](memory-types.md)
- [Hybrid retrieval and reranking](hybrid-retrieval-and-reranking.md)
- [Knowledge ingestion](knowledge-ingestion.md)
- [Defining agents](../agents-and-tools/defining-agents.md)
- [Workflow registry](../workflows/workflow-registry.md) (publish RAG chat)

## Source

Derived from [ARCFLOW-FULL-CAPABILITIES-REFERENCE.md](../../../docs/_draft/ARCFLOW-FULL-CAPABILITIES-REFERENCE.md) §6.2, §6.4; Appendix A (MemoryRetrievalConfig, MemoryChunkingConfig).
