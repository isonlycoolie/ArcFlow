# RAG benchmark methodology (Phase 2.5)

## Corpus

- Stub in-repo corpus for CI (`benchmarks/rag_nq_subset.py`).
- Optional `benchmarks/data/nq_subset_100.json` for a 100-query Natural Questions subset when available locally.

## Metrics

- **nDCG@3** — primary quality metric vs dense-only baseline.
- **Latency p50** — measure with Qdrant + embedding provider in a manual run (not enforced in stub script).

## Runs

```bash
python benchmarks/rag_nq_subset.py
```

Set `COHERE_API_KEY` to exercise live rerank in the runtime; the benchmark script documents fusion scores without remote calls.

## Exit criteria

Phase 2.5 claims hybrid retrieval when nDCG@3 is within 5% of a LlamaIndex reference on the same subset. Record results in `hybrid-rag-guide.md` when keys and Qdrant are available.
