# RAG chatbot example

**Audience:** `[developer]`

This walkthrough runs vector memory retrieval in a single-agent workflow. You configure hybrid dense and sparse retrieval, run a query, and confirm `MemoryRetrieved` events in the trace. The primary script is [`examples/rag/document_qa.py`](../../examples/rag/document_qa.py).

## What this example demonstrates

ArcFlow vector memory attaches to an agent via `MemoryConfig`. Chunking, embedding model selection, and hybrid retrieval weights live in workflow configuration. The sample uses stub embeddings for local runs; production swaps to real embedding providers and Qdrant.

A domain-heavy variant lives at [`examples/support/ticket_rag_bot.py`](../../examples/support/ticket_rag_bot.py) for support-ticket Q&A patterns.

## Prerequisites

| Item | Required for stub path | Required for full RAG |
|------|------------------------|----------------------|
| Python SDK | Yes | Yes |
| `ARCFLOW_QDRANT_URL` | No | Yes (e.g. `http://localhost:6333`) |
| Embedding provider env | No (uses `stub/8`) | Yes per [provider configuration](../guides/agents-and-tools/provider-configuration.md) |
| Docker Qdrant | No | Recommended via dev compose |
| Tutorial track | [Track C](../tutorials/track-c-rag.md) | Same |

## Step 1: Review memory configuration

From [`examples/rag/document_qa.py`](../../examples/rag/document_qa.py):

```python
from arcflow import Agent, MemoryChunkingConfig, MemoryConfig, MemoryRetrievalConfig, MemoryScope, MemoryType, Workflow

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
```

## Step 2: Run with stub embeddings

```bash
python examples/rag/document_qa.py
```

Stub mode exercises the workflow and trace path without Qdrant. Output includes a reminder to ingest documents when Qdrant is configured.

## Step 3: Run with Qdrant (optional full path)

Start Qdrant (dev stack or standalone), then export:

```bash
export ARCFLOW_QDRANT_URL=http://localhost:6333
# export embedding provider vars per your chosen model
python examples/rag/document_qa.py
```

Ingest the sample document in the script (`SAMPLE_DOC`) through vector ingest APIs or dashboard knowledge upload before expecting grounded answers. See [vector RAG pipeline](../guides/memory-and-rag/vector-rag-pipeline.md) and [knowledge ingestion](../guides/memory-and-rag/knowledge-ingestion.md).

## Step 4: Verify trace events

After `run()`, inspect events:

```python
result = workflow.run("Summarize hybrid retrieval in ArcFlow.")
kinds = {e.get("event_kind") for e in result.trace_events}
print(sorted(kinds))
if "MemoryRetrieved" in kinds:
    print("MemoryRetrieved present")
```

With populated vector store and matching namespace, expect `MemoryRetrieved` with chunk counts and scores (metadata only, no chunk text in trace export).

## Expected output

Stub run prints stub answer text plus:

```
(Ingest SAMPLE_DOC via vector APIs when Qdrant is configured.)
```

Full RAG run prints an answer grounded on ingested content. Pass criteria: non-empty `result.output`, `status == "completed"`, and `MemoryRetrieved` when the store contains matching namespace data.
