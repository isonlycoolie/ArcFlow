**Audience:** `[developer]` `[platform]`

# Hybrid retrieval and reranking

Hybrid retrieval combines dense vector similarity with sparse keyword signals in Qdrant. Optional Cohere reranking reorders candidates before the engine injects context into the agent prompt. Tuning weights and `top_k` / `top_n` balances recall, latency, and token cost.

Pipeline overview: [Vector RAG pipeline](vector-rag-pipeline.md). Memory types: [Memory types](memory-types.md).

## When to use hybrid

| Scenario | Recommendation |
|----------|----------------|
| Product docs with exact SKUs, error codes | Hybrid (sparse helps exact tokens) |
| Narrative FAQs, conceptual questions | Dense may suffice; hybrid still safe default |
| Short pages, small corpus | Lower `top_k`, skip rerank |

Hybrid requires `retrieval.mode: "hybrid"` **and** `ARCFLOW_QDRANT_HYBRID=true` at runtime.

## Retrieval config

```json
{
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
  }
}
```

| Field | Role |
|-------|------|
| `mode` | `"dense"` or `"hybrid"` |
| `top_k` | Candidates fetched from Qdrant before rerank |
| `dense_weight` | Weight for vector score in hybrid fusion |
| `sparse_weight` | Weight for sparse score in hybrid fusion |
| `rerank.top_n` | Final chunks after rerank (≤ `top_k`) |

Weights should reflect your corpus. Start with 0.65 / 0.35 and adjust using trace `chunk_count` and answer quality on a fixed eval set.

## Environment

```bash
export ARCFLOW_QDRANT_URL=http://localhost:6333
export ARCFLOW_QDRANT_HYBRID=true
export ARCFLOW_EMBEDDING_PROVIDER=openai
export OPENAI_API_KEY=sk-...
export COHERE_API_KEY=...
```

Without `ARCFLOW_QDRANT_HYBRID=true`, the engine may fall back to dense behavior even when config says hybrid. Verify in integration tests.

## Dense-only baseline

```json
{
  "retrieval": {
    "mode": "dense",
    "top_k": 6
  }
}
```

Compare latency and `MemoryRetrieved.total_bytes` against hybrid on the same queries before committing weights.

## Rerank stage

Cohere rerank receives query + chunk texts server-side (not in SEC-1 traces). Config:

```json
{
  "rerank": {
    "provider": "cohere",
    "model": "rerank-english-v3.0",
    "top_n": 3
  }
}
```

| Outcome | Error / trace |
|---------|---------------|
| Success | `MemoryRetrieved` with lower `chunk_count` |
| Provider failure | `RerankError`, HTTP 502 |
| Missing API key | Provider error at rerank call |

Disable rerank during dev to reduce dependencies:

```json
{
