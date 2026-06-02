
# Memory types

ArcFlow agents optionally attach `memory_config` to read and write state across a run or across runs. Four memory types cover scratch state, multi-agent handoff, durable key-value facts, and vector RAG. Scope controls isolation between agents, workflows, and individual runs.

Agent wiring: [Defining agents](../agents-and-tools/defining-agents.md). Vector details: [Vector RAG pipeline](vector-rag-pipeline.md).

## Memory types overview

| memory_type | Storage | Typical use |
|-------------|---------|-------------|
| `session` | In-run map | Scratch state within one execution |
| `shared` | In-run shared namespace | Multi-agent handoff in same workflow |
| `persistent` | PostgreSQL key-value | Cross-run facts per namespace |
| `vector` | Qdrant + embeddings | RAG, knowledge bases |

## Scope

| scope | Isolation |
|-------|-----------|
| `agent` | Private to one agent id |
| `workflow` | Shared across agents in the same workflow run |
| `run` | Scoped to a single run id |

Combine type and scope deliberately. Example: `shared` + `workflow` for pipeline state; `vector` + `workflow` for a knowledge namespace shared by support agents.

## Session memory

Ephemeral key-value within a single run. Lost when the run completes.

```json
{
 "memory_config": {
 "memory_type": "session",
 "scope": "agent",
 "namespace": "scratch"
 }
}
```

Use for counters, intermediate flags, or tool coordination within one agent step loop.

## Shared memory

Multiple agents in the same workflow read/write the same in-run namespace.

```json
{
 "memory_config": {
 "memory_type": "shared",
 "scope": "workflow",
 "namespace": "handoff"
 }
}
```

Pairs well with [Context policies](../agents-and-tools/context-policies.md) when prior step text truncation is too lossy.

## Persistent memory

Facts survive across runs when Postgres is available.

```json
{
 "memory_config": {
 "memory_type": "persistent",
 "scope": "workflow",
 "namespace": "customer-acme-4421",
 "ttl_seconds": 604800
 }
}
```

Requires server or SDK run with Postgres configured. Failures surface as `MemoryError` (HTTP 503).

## Vector memory

Semantic retrieval from Qdrant. Requires embedding provider and Qdrant URL in production.

```json
{
 "memory_config": {
 "memory_type": "vector",
 "scope": "workflow",
 "namespace": "product-docs",
 "embedding": "openai/text-embedding-3-small",
 "retrieval": {
 "mode": "hybrid",
 "top_k": 8
 },
 "chunking": {
 "chunk_size": 512,
 "chunk_overlap": 64
 }
 }
}
```

See [Vector RAG pipeline](vector-rag-pipeline.md), [Hybrid retrieval and reranking](hybrid-retrieval-and-reranking.md), and [Knowledge ingestion](knowledge-ingestion.md).

## Trace events

| Event | Fields of note |
|-------|----------------|
| `MemoryWrite` | memory_type, key, duration_ms |
| `MemoryRead` | hit (boolean), key |
| `MemoryRetrieved` | chunk_count, total_bytes (vector only) |
| `MemoryDegraded` | backend, reason |
| `MemoryEvicted` | key, eviction_reason |

trace data policy: no chunk text or values in traces. [Trace data policy](../../concepts/sec-1-and-data-safety.md).

Example vector retrieval trace:

```json
{
 "kind": "MemoryRetrieved",
 "run_id": "r1",
 "step_id": "s1",
 "agent_name": "researcher",
 "chunk_count": 5,
 "total_bytes": 3840
}
```

## Degradation behavior

| Event | Meaning |
|-------|---------|
| `MemoryDegraded` | Backend unavailable; engine may continue with reduced capability |
| `MemoryEvicted` | TTL or capacity eviction |
| `MemoryRead` hit=false | Key miss; agent proceeds without that context |

Production vector setups need `ARCFLOW_QDRANT_URL` and non-stub `ARCFLOW_EMBEDDING_PROVIDER`. Stub embedding is dev-only.

## Environment variables

| Variable | Purpose |
|----------|---------|
| `ARCFLOW_QDRANT_URL` | Qdrant endpoint |
| `ARCFLOW_EMBEDDING_PROVIDER` | Embedding backend |
| `ARCFLOW_QDRANT_HYBRID` | Enable hybrid dense+sparse search |
| `ARCFLOW_POSTGRESQL_URL` | Persistent memory and server runs |
| `OPENAI_API_KEY` | Embeddings when using OpenAI embedding strings |

## Choosing a type

| Need | Choice |
|------|--------|
| Single agent scratch pad | `session` / `agent` |
| Pass structured state between agents | `shared` / `workflow` |
| Remember user prefs next week | `persistent` / `workflow` + namespace |
| Answer from documentation | `vector` + ingest pipeline |

## Related pages

- [Vector RAG pipeline](vector-rag-pipeline.md)
- [Knowledge ingestion](knowledge-ingestion.md)
- [Provider configuration](../agents-and-tools/provider-configuration.md)
- [Install and build](../../getting-started/install-and-build.md)
