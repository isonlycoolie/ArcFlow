# 03 Retrieval and agent wiring


## Before you start

Complete [02 Ingest documents](02-ingest-documents.md) so namespace `doc_qa` (or your chosen id) contains at least one document. Qdrant must be running and `ARCFLOW_QDRANT_URL` exported.

## Concept

Retrieval is automatic once an agent has vector memory config and the namespace holds embedded chunks. You do not call `VectorStore.search` inside the workflow for the default agent path; the runtime queries Qdrant during step execution and injects results into the prompt.

Wiring checklist:

1. **Ingest** into namespace `N` with `VectorStore.ingest(N, key, text)`.
2. **Configure** agent with `MemoryConfig(MemoryType.VECTOR, ..., namespace=N, embedding=...)`.
3. **Instruct** the agent to use retrieved context (e.g. "Answer using retrieved context.").
4. **Run** `workflow.run(user_question)` and inspect `result.output` and `result.trace_events`.

Trace event `MemoryRetrieved` confirms retrieval ran. Payload fields include `chunk_count` and `total_bytes`, not chunk text (SEC-1).

[RAG chatbot walkthrough](../../examples/rag-chatbot.md) on the current branch shows steps 2 through 4; ingest is left as a comment. The restructure branch `memory_guide_qa.py` runs ingest and query in one `main()`.

## Example

End-to-end script combining ingest and query:

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
from arcflow.memory import VectorStore

NAMESPACE = "doc_qa"
SAMPLE_DOC = """ArcFlow vector memory supports hybrid dense and sparse retrieval.
Chunk documents before ingest for better recall on long texts."""

def main() -> None:
    store = VectorStore()
    store.ingest(NAMESPACE, "guide", SAMPLE_DOC)

    memory = MemoryConfig(
        MemoryType.VECTOR,
        MemoryScope.AGENT,
        namespace=NAMESPACE,
        embedding="stub/8",
        retrieval=MemoryRetrievalConfig(mode="hybrid", top_k=3),
        chunking=MemoryChunkingConfig(chunk_size=256, overlap=32),
    )
    agent = Agent(
        name="researcher",
        role="researcher",
        instructions="Answer using retrieved context. If context is empty, say so.",
        memory=memory,
    )
    workflow = Workflow(name="document-qa", agents=[agent])
    result = workflow.run("What chunking advice does the guide give?")
    print(result.output)
    print(f"status={result.status} run_id={result.run_id}")

    kinds = {e.get("event_kind") for e in result.trace_events}
    print("trace events:", sorted(kinds))

if __name__ == "__main__":
    main()
```

Run:

```bash
export ARCFLOW_QDRANT_URL=http://localhost:6333
python retrieval_wiring.py
```

Compare with the split layout in `document_qa.py` (query only) versus `memory_guide_qa.py` (ingest file then query on restructure branch).

## Verify

| Check | Expected |
|-------|----------|
| `result.status` | `"completed"` |
| `result.output` | Non-empty string |
| `MemoryRetrieved` in trace kinds | Present when namespace populated and Qdrant reachable |
| Wrong namespace on agent | No retrieval hits; answer may ignore ingested facts |

Assertion used in [Track C](../../tutorials/track-c-rag.md):

```python
kinds = {e.get("event_kind") for e in result.trace_events}
assert "MemoryRetrieved" in kinds, f"missing MemoryRetrieved; got {kinds}"
```

With a live LLM provider, answers should reference ingested facts. With stub provider, focus on trace presence and completed status.

## Next

[04 Hybrid retrieval intro](04-hybrid-retrieval-intro.md) explains dense versus hybrid mode and environment flags.
