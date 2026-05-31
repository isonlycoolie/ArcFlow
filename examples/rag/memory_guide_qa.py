# Internal memory guide Q&A — hybrid retrieval with chunking over platform docs.

from __future__ import annotations

import sys
from pathlib import Path

from arcflow import Agent, MemoryChunkingConfig, MemoryConfig, MemoryRetrievalConfig, MemoryScope, MemoryType, VectorStore, Workflow

NAMESPACE = "platform-docs-memory-guide"
GUIDE_PATH = Path(__file__).parent / "data" / "memory_guide.md"
DEFAULT_QUESTION = "When should we use hybrid retrieval and what chunk size for API docs?"


def main() -> None:
    question = sys.argv[1] if len(sys.argv) > 1 else DEFAULT_QUESTION
    doc = GUIDE_PATH.read_text(encoding="utf-8")

    store = VectorStore()
    store.ingest(NAMESPACE, "memory_guide", doc)

    memory = MemoryConfig(
        MemoryType.VECTOR,
        MemoryScope.AGENT,
        namespace=NAMESPACE,
        embedding="stub/384",
        retrieval=MemoryRetrievalConfig(
            mode="hybrid",
            dense_weight=0.65,
            sparse_weight=0.35,
            rerank="local",
            top_k=4,
        ),
        chunking=MemoryChunkingConfig(chunk_size=256, overlap=32),
    )
    agent = Agent(
        name="platform_doc_assistant",
        role="platform_engineer",
        instructions="Answer using retrieved guide sections only. Cite chunk topics, not URLs.",
        memory=memory,
    )
    workflow = Workflow(name="memory_guide_qa").step(agent)
    result = workflow.run(question)
    print(result.output)
    print(f"\nrun_id={result.run_id}")


if __name__ == "__main__":
    main()
