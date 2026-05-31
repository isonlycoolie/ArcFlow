# 02 Ingest documents

**Audience:** `[developer]`

## Before you start

Read [01 RAG concepts in ArcFlow](01-rag-concepts-in-arcflow.md). Export Qdrant URL before ingest:

```bash
export ARCFLOW_QDRANT_URL=http://localhost:6333
```

Stub embedding works without `ARCFLOW_EMBEDDING_PROVIDER`. Production ingest should use a real embedding string and matching provider env vars.

## Concept

`VectorStore` lives in `arcflow.memory` (not re-exported from the top-level `arcflow` package on all branches). Construct it once, then call `ingest` per document:

```python
from arcflow.memory import VectorStore

store = VectorStore()
chunk_count = store.ingest(namespace, key, text)  # returns int
```

| Argument | Role |
|----------|------|
| `namespace` | Qdrant collection key; must match agent `MemoryConfig.namespace` |
| `key` | Logical document id (replace content by re-ingesting with the same key) |
| `text` | Raw string to chunk and embed |

The return value is the number of chunks written. Empty namespace raises `MemoryConfigurationError`.

Ingest is separate from `Workflow.run()`. Typical patterns:

1. **Setup script**: ingest in `main()` before creating the workflow (see `examples/support/ticket_rag_bot.py`).
2. **One-shot demo**: read a file, ingest, then run the query in the same process (see `memory_guide_qa.py` on the restructure branch).

Chunking defaults come from the runtime when ingest is called without per-call overrides. Set `MemoryChunkingConfig` on the agent so retrieval expects the same segment sizes you used conceptually at ingest time.

## Example

Save as `ingest_demo.py`:

```python
from arcflow.memory import VectorStore

NAMESPACE = "doc_qa"
KEY = "memory-guide"

SAMPLE_DOC = """# ArcFlow Memory Guide

ArcFlow vector memory supports hybrid dense and sparse retrieval.
Chunk documents before ingest for better recall on long texts.
Optional rerank improves top-k precision on noisy corpora.
"""

def main() -> None:
    store = VectorStore()
    chunks = store.ingest(NAMESPACE, KEY, SAMPLE_DOC)
    print(f"ingested {chunks} chunks into namespace={NAMESPACE!r} key={KEY!r}")

    hits = store.search(NAMESPACE, "hybrid retrieval", top_k=2)
    for i, hit in enumerate(hits, start=1):
        print(f"hit {i}: {hit.byte_len} bytes, preview={hit.text[:80]!r}...")

if __name__ == "__main__":
    main()
```

Run:

```bash
export ARCFLOW_QDRANT_URL=http://localhost:6333
python ingest_demo.py
```

File-based ingest (pattern from restructure branch `memory_guide_qa.py`):

```python
from pathlib import Path
from arcflow.memory import VectorStore

NAMESPACE = "platform-docs-memory-guide"
doc = Path("examples/rag/data/memory_guide.md").read_text(encoding="utf-8")
store = VectorStore()
print(store.ingest(NAMESPACE, "memory_guide", doc))
```

Use the same `NAMESPACE` on the agent in that branch's script.

Domain examples that ingest before run:

| Example | Namespace | Key |
|---------|-----------|-----|
| `examples/support/ticket_rag_bot.py` | `support-tickets` | `kb` |
| `examples/education/course_qa.py` | `course-101` | `syllabus` |
| `examples/healthcare/protocol_qa.py` | `clinical-kb` | `protocols` |

## Verify

| Check | Expected |
|-------|----------|
| `store.ingest("", "k", "text")` | `MemoryConfigurationError` |
| Successful ingest | Positive integer chunk count |
| `store.search(NAMESPACE, query, top_k=2)` after ingest | Non-empty list when Qdrant is up and namespace matches |

Re-ingesting with the same `key` replaces that document's chunks rather than duplicating under a new id.

## Next

[03 Retrieval and agent wiring](03-retrieval-and-agent-wiring.md) connects ingested namespaces to agents and verifies `MemoryRetrieved` on query runs.

## Source

`sdk-python/arcflow/memory.py` (`VectorStore.ingest`, `VectorStore.search`); [`examples/rag/document_qa.py`](../../../examples/rag/document_qa.py); `examples/rag/memory_guide_qa.py` (restructure branch); [`examples/support/ticket_rag_bot.py`](../../../examples/support/ticket_rag_bot.py).
