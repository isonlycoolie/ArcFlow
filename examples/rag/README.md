# Document Q&A over Internal Guides

## Problem

Your platform team maintains a 40-page **Memory & Retrieval guide** in Markdown. New engineers ask the same questions in Slack:

- How does hybrid retrieval weighting work?
- When should we enable rerank?
- What chunk size should we use for API docs?

Copy-pasting doc sections does not scale. The team wants **grounded Q&A** with chunking tuned for long technical prose.

## Who this is for

| Role | Goal |
|------|------|
| **Platform engineer** | Self-serve answers from internal docs |
| **Technical writer** | Validate chunking/retrieval settings before production ingest |

## What ArcFlow demonstrates

- `MemoryChunkingConfig` (256 tokens, 32 overlap)
- **Hybrid** dense/sparse retrieval with local rerank stub
- Agent-scoped vector namespace

## Prerequisites

- Python SDK installed
- Production path: Qdrant + `ARCFLOW_QDRANT_URL` + real embeddings

## Run

```bash
python examples/rag/memory_guide_qa.py
```

## Verify

- Output mentions hybrid retrieval or chunking from [`data/memory_guide.md`](data/memory_guide.md)
- Script prints reminder to ingest via vector APIs when Qdrant is configured
- Trace includes `MemoryRetrieved` when ingest path is wired

## Production notes

- Ingest from CI when docs change; version namespaces (`docs-memory-v2`)
- Enable Cohere rerank with `COHERE_API_KEY` for long corpora
- See [support/](../support/) for hybrid + persistent namespace pattern

## Files

| File | Purpose |
|------|---------|
| [`memory_guide_qa.py`](memory_guide_qa.py) | Chunking + hybrid retrieval demo |
| [`data/memory_guide.md`](data/memory_guide.md) | Sample internal guide |
