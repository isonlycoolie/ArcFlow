# Track C: RAG and vector memory

**Audience:** `[developer]`

Track C walks through document ingestion, vector memory configuration, and query runs that emit `MemoryRetrieved` trace events. You use Qdrant plus embedding configuration (stub or real provider).

## Goal

Ingest documents, query via a vector-memory agent, and verify retrieval in execution traces. First contact with Qdrant, embedding env vars, and hybrid retrieval settings.

## Prerequisites

| Item | Required |
|------|----------|
| [Track A](track-a-first-workflow.md) | Completed or equivalent SDK familiarity |
| Python SDK | Built |
| Qdrant | `ARCFLOW_QDRANT_URL` (e.g. dev stack on port 6333) |
| Primary example | [RAG chatbot example](../examples/rag-chatbot.md) |
| Script | [`examples/rag/document_qa.py`](../../examples/rag/document_qa.py) |
| Guides | [Vector RAG pipeline](../guides/memory-and-rag/vector-rag-pipeline.md), [knowledge ingestion](../guides/memory-and-rag/knowledge-ingestion.md) |

Stub-only path works without Qdrant but will not emit `MemoryRetrieved` until documents exist in the store.

## Step 1: Start Qdrant (recommended)

If your dev compose includes Qdrant, start it with the server stack or standalone container on port 6333. Export:

```bash
export ARCFLOW_QDRANT_URL=http://localhost:6333
```

## Step 2: Configure memory on an agent

Use the sample from [`document_qa.py`](../../examples/rag/document_qa.py):

```python
memory = MemoryConfig(
    MemoryType.VECTOR,
    MemoryScope.AGENT,
    namespace="track_c_kb",
    embedding="stub/8",  # swap for production embedding id
    retrieval=MemoryRetrievalConfig(
        mode="hybrid",
        dense_weight=0.7,
        sparse_weight=0.3,
        top_k=3,
    ),
    chunking=MemoryChunkingConfig(chunk_size=256, overlap=32),
)
agent = Agent(name="researcher", role="researcher", instructions="Answer using retrieved context.", memory=memory)
workflow = Workflow(name="track-c-rag", agents=[agent])
```

Change namespace to a dedicated track id to avoid colliding with other tutorials.

## Step 3: Ingest sample documents

Ingest text into namespace `track_c_kb` using admin ingest API, dashboard Knowledge tab, or SDK ingest helpers described in [knowledge ingestion](../guides/memory-and-rag/knowledge-ingestion.md).

Minimal text for verification:

```
ArcFlow hybrid retrieval combines dense vectors with sparse signals.
Chunk size and overlap affect recall on long documents.
```

Confirm ingest success before querying.

## Step 4: Run query

```bash
python examples/rag/document_qa.py
```

Or inline:

```python
result = workflow.run("What does ArcFlow use for hybrid retrieval?")
print(result.output)
```

## Step 5: Verify trace events

```python
kinds = {e.get("event_kind") for e in result.trace_events}
assert "MemoryRetrieved" in kinds, f"missing MemoryRetrieved; got {kinds}"
assert result.status == "completed"
print("track C trace checks passed")
```

Inspect `MemoryRetrieved` payloads for chunk counts and scores without chunk text (SEC-1).

## Verification checklist

| Check | Expected |
|-------|----------|
| Ingest | Namespace populated |
| `result.status` | `completed` |
| `MemoryRetrieved` | Present on query run |
| `WorkflowCompleted` | Present |
| Answers | Reference ingested facts when using real provider |

## Expected output

Non-empty answer string referencing hybrid retrieval concepts when the store is populated. Verification prints `track C trace checks passed`.

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| No `MemoryRetrieved` | No ingest or wrong namespace | Re-ingest into `track_c_kb` |
| Qdrant connection refused | Service not running | Start Qdrant; verify URL |
| Generic stub answer | Stub path without store | Expected until ingest completes |
| Poor recall | Chunk settings | Tune `chunk_size` and `overlap` per guide |

## What you learned

Track C connects agent memory configuration to operational ingest and trace-verified retrieval. Platform RAG features in static product and server admin APIs build on the same vector primitives.

## Next tracks

| Track | Focus |
|-------|-------|
| D | Graph routing |
| F | Static product with dashboard-ingested knowledge |
| Level 2 cert | RAG plus graph plus HITL combined project |

**Source:** capabilities reference §28 Track C; [`examples/rag/document_qa.py`](../../examples/rag/document_qa.py); [RAG chatbot example](../examples/rag-chatbot.md); [vector RAG pipeline](../guides/memory-and-rag/vector-rag-pipeline.md).
