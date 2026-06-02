# 03 Vector memory setup


## Before you start

Complete [01 Memory types overview](01-memory-types-overview.md). You should understand that `MemoryType.VECTOR` requires a `namespace` and usually an `embedding` string. For ingest and query together, read [02 Ingest documents](../rag/02-ingest-documents.md) in the RAG track after this page.

## Concept

Vector memory stores embedded document chunks in Qdrant. At run time the runtime queries the collection named by your `namespace`, retrieves top chunks, and injects them into the agent prompt. Configuration lives on the agent:

```python
memory = MemoryConfig(
 MemoryType.VECTOR,
 MemoryScope.AGENT,
 namespace="doc_qa", # must match VectorStore.ingest namespace
 embedding="stub/8", # local dev; swap for production embedding id
 retrieval=MemoryRetrievalConfig(mode="hybrid", top_k=3),
 chunking=MemoryChunkingConfig(chunk_size=256, overlap=32),
)
```

### Environment variables

| Variable | Purpose |
|----------|---------|
| `ARCFLOW_QDRANT_URL` | Qdrant HTTP endpoint (e.g. `http://localhost:6333`) |
| `ARCFLOW_EMBEDDING_PROVIDER` | Embedding backend for non-stub models |
| `ARCFLOW_QDRANT_HYBRID` | Set to `true` when using hybrid retrieval mode |
| `OPENAI_API_KEY` | Required when embedding string uses OpenAI models |

Stub embedding (`stub/8`, `stub/384`, etc.) lets you exercise config and workflow wiring without a real embedding provider. Qdrant is still required if you want chunks persisted and `MemoryRetrieved` events on query runs.

### Namespace rules

The `namespace` on `MemoryConfig` must match the first argument passed to `VectorStore.ingest(namespace, key, text)`. A mismatch produces empty retrieval and generic stub answers even when documents exist in another collection.

Pick one namespace per knowledge base or tutorial (e.g. `doc_qa`, `track_c_kb`, `platform-docs-memory-guide`) and use it consistently for ingest and agent config.

## Example

This mirrors [RAG chatbot walkthrough](../../examples/rag-chatbot.md) on the current branch. On the examples restructure branch the full ingest-plus-query flow is in `examples/rag/memory_guide_qa.py`.

Save as `vector_memory_setup.py`:

```python
from arcflow import (
 Agent,
 MemoryChunkingConfig,
 MemoryConfig,
 MemoryRetrievalConfig,
 MemoryScope,
 MemoryType,
 Workflow,
)

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

agent = Agent(
 name="researcher",
 role="researcher",
 instructions="Answer using retrieved context.",
 memory=memory,
)

workflow = Workflow(name="document-qa", agents=[agent])
result = workflow.run("Summarize hybrid retrieval in ArcFlow.")
print(result.output)
print(f"run_id={result.run_id} status={result.status}")
```

Start Qdrant (dev compose or standalone container), then export the URL before ingest and query:

```bash
export ARCFLOW_QDRANT_URL=http://localhost:6333
python vector_memory_setup.py
```

Ingest sample text into the same namespace before expecting grounded answers:

```python
from arcflow.memory import VectorStore

SAMPLE_DOC = """ArcFlow vector memory supports hybrid dense and sparse retrieval.
Chunk documents before ingest for better recall on long texts."""

store = VectorStore()
chunk_count = store.ingest("doc_qa", "memory-guide", SAMPLE_DOC)
print(f"ingested {chunk_count} chunks")
```

Run ingest once, then run the workflow query again.

## Verify

| Check | Expected |
|-------|----------|
| `MemoryConfig` without `namespace` | `MemoryConfigurationError` |
| Workflow run with stub embedding | `status == "completed"`, non-empty `result.output` |
| After ingest with matching namespace | `MemoryRetrieved` in `result.trace_events` when Qdrant is up |

Trace check:

```python
kinds = {e.get("event_kind") for e in result.trace_events}
assert "MemoryRetrieved" in kinds, f"missing MemoryRetrieved; got {kinds}"
```

trace data policy: trace payloads include chunk counts and byte totals, not chunk text.

## Next

Continue to the [RAG track](../rag/README.md) for ingest patterns, agent wiring, and hybrid retrieval tuning.
