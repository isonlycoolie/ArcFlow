"""Document Q&A with hybrid vector memory (Phase 2.5)."""

from __future__ import annotations

from arcflow import Agent, MemoryChunkingConfig, MemoryConfig, MemoryRetrievalConfig, MemoryScope, MemoryType, Workflow

SAMPLE_DOC = """# ArcFlow Memory Guide
Source: https://arcflow.dev/docs/memory

ArcFlow vector memory supports hybrid dense and sparse retrieval.
Chunk documents before ingest for better recall on long texts.
Optional Cohere rerank improves top-k precision when COHERE_API_KEY is set.
"""


def main() -> None:
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
    print("(Ingest SAMPLE_DOC via vector APIs when Qdrant is configured.)")


if __name__ == "__main__":
    main()
