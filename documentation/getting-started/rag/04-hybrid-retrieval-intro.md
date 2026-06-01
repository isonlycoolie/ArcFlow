# 04 Hybrid retrieval intro


## Before you start

Complete [03 Retrieval and agent wiring](03-retrieval-and-agent-wiring.md) so you have seen `MemoryRetrieved` on a populated namespace. Read [RAG chatbot example](../../examples/rag-chatbot.md) hybrid settings before tuning weights.

## Concept

Hybrid retrieval combines **dense** vector similarity with **sparse** keyword signals in Qdrant. Dense-only mode (`mode="dense"`) uses embeddings alone. Hybrid mode (`mode="hybrid"`) fuses both scores using configurable weights before optional reranking.

Python SDK shape:

```python
MemoryRetrievalConfig(
    mode="hybrid",           # or "dense"
    dense_weight=0.7,
    sparse_weight=0.3,
    rerank="local",          # optional: "local", "cohere", or None
    top_k=3,
)
```

| Field | Role |
|-------|------|
| `mode` | `"dense"` or `"hybrid"` |
| `dense_weight` | Weight for embedding similarity in hybrid fusion |
| `sparse_weight` | Weight for sparse term signals in hybrid fusion |
| `top_k` | Candidate chunks fetched before rerank |
| `rerank` | Reorder top candidates; `"local"` needs no Cohere key |

Hybrid also requires runtime support: set `ARCFLOW_QDRANT_HYBRID=true` when Qdrant is configured for sparse vectors. Without it, the engine may behave like dense search even when config says hybrid. Verify with trace `chunk_count` and answer quality on fixed test questions.

When to prefer hybrid:

| Corpus | Reason |
|--------|--------|
| Docs with SKUs, error codes, API paths | Sparse helps exact token matches |
| Long narrative guides | Dense helps paraphrased questions; hybrid is a safe default |
| Small FAQ set | Lower `top_k`; rerank optional |

Stub embedding (`stub/8`) still runs hybrid config through the workflow for local wiring tests. Meaningful ranking quality needs real embeddings and populated Qdrant data.

## Example

From [RAG chatbot example](../../examples/rag-chatbot.md):

```python
from arcflow import MemoryChunkingConfig, MemoryConfig, MemoryRetrievalConfig, MemoryScope, MemoryType

memory = MemoryConfig(
    MemoryType.VECTOR,
    MemoryScope.AGENT,
    namespace="doc_qa",
    embedding="stub/8",
    retrieval=MemoryRetrievalConfig(
        mode="hybrid",
        dense_weight=0.7,
        sparse_weight=0.3,
        rerank="local",
        top_k=3,
    ),
    chunking=MemoryChunkingConfig(chunk_size=256, overlap=32),
)
```

Environment for a local hybrid run:

```bash
export ARCFLOW_QDRANT_URL=http://localhost:6333
export ARCFLOW_QDRANT_HYBRID=true
# Run your ingest + query script (see RAG chatbot example walkthrough)
python your_rag_demo.py
```

Dense baseline for comparison:

```python
retrieval=MemoryRetrievalConfig(mode="dense", top_k=6)
```

Run the same question against both configs on an ingested namespace and compare latency plus `MemoryRetrieved` metadata. Start near 0.65 / 0.35 weights if 0.7 / 0.3 recall is weak on keyword-heavy queries.

## Verify

| Check | Expected |
|-------|----------|
| Invalid `mode` (e.g. `"sparse"`) | `MemoryConfigurationError` at config time |
| Hybrid with `ARCFLOW_QDRANT_HYBRID=true` and ingest done | `MemoryRetrieved` with `chunk_count` ≤ `top_k` (before rerank expansion) |
| `rerank="cohere"` without `COHERE_API_KEY` | Runtime error or degraded path depending on version; prefer `"local"` locally |

Track C troubleshooting: if answers ignore ingested facts, confirm namespace match and hybrid env flag before changing weights.

## Next

| Goal | Document |
|------|----------|
| Full tutorial checklist | [Track C: RAG and vector memory](../../tutorials/track-c-rag.md) |
| Weight and rerank tuning | [Hybrid retrieval and reranking](../../guides/memory-and-rag/hybrid-retrieval-and-reranking.md) |
| Operator ingest via admin API | [Knowledge ingestion](../../guides/memory-and-rag/knowledge-ingestion.md) |
