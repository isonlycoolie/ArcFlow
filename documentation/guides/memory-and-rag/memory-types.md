**Audience:** `[developer]` `[platform]`

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
