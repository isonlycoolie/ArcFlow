# RAG track


This track walks from RAG concepts in ArcFlow through document ingest, agent wiring, and a first look at hybrid retrieval. You use `VectorStore.ingest`, attach `MemoryConfig` to agents, and verify retrieval through workflow traces.

## What you will learn

| Lesson | Topic |
|--------|-------|
| [01 RAG concepts in ArcFlow](01-rag-concepts-in-arcflow.md) | Ingest, embed, retrieve, inject; how namespace ties the pipeline together |
| [02 Ingest documents](02-ingest-documents.md) | `VectorStore.ingest(namespace, key, text)` and chunk counts |
| [03 Retrieval and agent wiring](03-retrieval-and-agent-wiring.md) | Query runs, `MemoryRetrieved` events, end-to-end Q&A |
| [04 Hybrid retrieval intro](04-hybrid-retrieval-intro.md) | Dense plus sparse weights, `ARCFLOW_QDRANT_HYBRID`, rerank basics |

## Prerequisites

Complete the [Memory track](../memory/README.md), especially [03 Vector memory setup](../memory/03-vector-memory-setup.md). You need the Python SDK built ([Install and build](../install-and-build.md)) and Qdrant reachable at `ARCFLOW_QDRANT_URL` for the full ingest-and-query path. Stub embedding (`stub/8` or `stub/384`) is enough for local dev without a paid embedding API.

Primary walkthrough: [RAG chatbot example](../../examples/rag-chatbot.md).

## How these lessons are structured

Every page follows the same sections: **Before you start**, **Concept**, **Example**, **Verify**, and **Next**.

## After this track

| Goal | Next document |
|------|---------------|
| Tutorial with verification checklist | [Track C: RAG and vector memory](../../tutorials/track-c-rag.md) |
| RAG chatbot walkthrough | [RAG chatbot example](../../examples/rag-chatbot.md) |
| Hybrid tuning deep dive | [Hybrid retrieval and reranking](../../guides/memory-and-rag/hybrid-retrieval-and-reranking.md) |
