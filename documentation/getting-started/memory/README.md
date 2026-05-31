# Memory track


This track teaches how ArcFlow agents attach memory backends through `MemoryConfig`. You start with the four memory types and scope rules, then practice session and shared memory without external services, and finish with vector memory setup for Qdrant-backed RAG.

## What you will learn

| Lesson | Topic |
|--------|-------|
| [01 Memory types overview](01-memory-types-overview.md) | `MemoryType`, `MemoryScope`, and when to pick each combination |
| [02 Session and shared memory](02-session-and-shared-memory.md) | In-run scratch state and multi-agent handoff |
| [03 Vector memory setup](03-vector-memory-setup.md) | Qdrant via `ARCFLOW_QDRANT_URL`, namespace, stub embeddings |

## Prerequisites

Complete [Install and build](../install-and-build.md) so `from arcflow import Agent, MemoryConfig, MemoryScope, MemoryType, Workflow` works in your virtual environment. Read [03 Anatomy of a workflow](../fundamentals/03-anatomy-of-a-workflow.md) and [Writing agents](../writing-agents/README.md) so agent construction and step wiring are already familiar.

Lessons 01 and 02 run with the stub provider and no Docker. Lesson 03 adds Qdrant for vector storage; stub embedding (`stub/8` or `stub/384`) keeps local runs working without a real embedding API key.

## How these lessons are structured

Every page follows the same sections: **Before you start**, **Concept**, **Example**, **Verify**, and **Next**. Run each example as a standalone script unless the page says otherwise.

## After this track

| Goal | Next document |
|------|---------------|
| Ingest documents and wire RAG agents | [RAG track](../rag/README.md) |
| Guided verification with trace events | [Track C: RAG and vector memory](../../tutorials/track-c-rag.md) |
| Full memory reference | [Memory types](../../guides/memory-and-rag/memory-types.md) |
