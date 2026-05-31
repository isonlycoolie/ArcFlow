# 01 RAG concepts in ArcFlow

**Audience:** `[developer]`

## Before you start

Read [03 Vector memory setup](../memory/03-vector-memory-setup.md) and [01 Memory types overview](../memory/01-memory-types-overview.md). You should know that vector agents use `MemoryType.VECTOR` and a required `namespace`.

## Concept

RAG in ArcFlow is not a separate product feature. It is vector memory used in two phases: **ingest** (write) and **run** (read).

```text
Source text → chunk → embed → upsert Qdrant (namespace)
User query → embed query → search namespace → inject chunks → agent LLM
```

**Ingest** happens outside the workflow run (or in a setup block before `run()`). The Python SDK exposes `VectorStore.ingest(namespace, key, text)`. The runtime chunks the text, embeds each chunk, and stores vectors under the namespace collection in Qdrant.

**Retrieval** happens when a workflow step executes an agent that carries `MemoryConfig` with `MemoryType.VECTOR`. The runtime searches the same namespace, ranks chunks, and adds them to the agent context before the model generates an answer.

Three names must stay aligned:

| Name | Where it appears |
|------|------------------|
| `namespace` | `MemoryConfig(..., namespace="doc_qa")` and `VectorStore.ingest("doc_qa", ...)` |
| `embedding` | Agent memory config; ingest uses the runtime default embedding unless you configure otherwise |
| `key` | Logical document id in `ingest(namespace, key, text)` for updates and replacement |

Chunking can be set on the agent via `MemoryChunkingConfig`. Align chunk size and overlap with how you ingest long documents so retrieval sees consistent segment boundaries.

For local development, `embedding="stub/8"` (or `stub/384`) avoids API keys. Set `ARCFLOW_QDRANT_URL=http://localhost:6333` when you want real persistence and `MemoryRetrieved` trace events.

The canonical query-side sample on the current branch is [`examples/rag/document_qa.py`](../../../examples/rag/document_qa.py). The restructure branch combines ingest and query in `examples/rag/memory_guide_qa.py`.

## Example

Read the sample document constant from `document_qa.py`:

```python
SAMPLE_DOC = """# ArcFlow Memory Guide
Source: https://arcflow.dev/docs/memory

ArcFlow vector memory supports hybrid dense and sparse retrieval.
Chunk documents before ingest for better recall on long texts.
Optional Cohere rerank improves top-k precision when COHERE_API_KEY is set.
"""
```

That text is meant to be ingested into namespace `doc_qa` before the workflow asks about hybrid retrieval. The agent config in the same file sets:

```python
memory = MemoryConfig(
    MemoryType.VECTOR,
    MemoryScope.AGENT,
    namespace="doc_qa",
    embedding="stub/8",
    retrieval=MemoryRetrievalConfig(mode="hybrid", dense_weight=0.7, sparse_weight=0.3, top_k=3),
    chunking=MemoryChunkingConfig(chunk_size=256, overlap=32),
)
```

Run the script without ingest to see stub workflow behavior. Run ingest first to see retrieval-backed answers when Qdrant is configured.

## Verify

| Check | Expected |
|-------|----------|
| `document_qa.py` imports | `MemoryConfig`, `MemoryType`, `MemoryScope`, `MemoryRetrievalConfig`, `MemoryChunkingConfig` |
| Namespace in script | `"doc_qa"` |
| Stub embedding in script | `"stub/8"` |

After you complete [02 Ingest documents](02-ingest-documents.md), re-run a query and confirm `MemoryRetrieved` appears in traces.

## Next

[02 Ingest documents](02-ingest-documents.md) covers `VectorStore.ingest` with a runnable ingest script.

## Source

[`examples/rag/document_qa.py`](../../../examples/rag/document_qa.py); `examples/rag/memory_guide_qa.py` (restructure branch); [Knowledge ingestion](../../guides/memory-and-rag/knowledge-ingestion.md); [Vector RAG pipeline](../../guides/memory-and-rag/vector-rag-pipeline.md).
