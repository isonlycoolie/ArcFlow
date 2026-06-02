# 01 Memory types overview


## Before you start

You should have the Python SDK installed ([Install and build](../install-and-build.md#python-sdk-maturin--pip)) and have read [02 Anatomy of an agent](../fundamentals/02-anatomy-of-an-agent.md). Memory attaches to an agent through the optional `memory=` argument on `Agent(...)`. The runtime reads that config when the step executes.

## Concept

ArcFlow memory is configured with two enums from `arcflow`:

| Enum | Values | Role |
|------|--------|------|
| `MemoryType` | `SESSION`, `SHARED`, `PERSISTENT`, `VECTOR` | Which storage backend the agent uses |
| `MemoryScope` | `AGENT`, `WORKFLOW`, `GLOBAL` | Who can read and write within a run |

`MemoryConfig` wraps both plus optional fields:

```python
MemoryConfig(
 memory_type, # MemoryType enum (required)
 scope=MemoryScope.AGENT,
 namespace=None, # required for PERSISTENT and VECTOR
 ttl_seconds=None, # optional expiry for persistent keys
 embedding=None, # vector only, e.g. "stub/8" or "openai/text-embedding-3-small"
 retrieval=None, # MemoryRetrievalConfig for vector
 chunking=None, # MemoryChunkingConfig for vector ingest alignment
)
```

### MemoryType at a glance

| `MemoryType` | Backend | Typical use |
|--------------|---------|-------------|
| `SESSION` | In-process map for one run | Scratch counters, flags, tool coordination |
| `SHARED` | In-run shared namespace | Pass structured state between agents in the same workflow |
| `PERSISTENT` | PostgreSQL key-value | Facts that survive across runs (needs `ARCFLOW_POSTGRESQL_URL`) |
| `VECTOR` | Qdrant plus embeddings | RAG, documentation Q&A (needs `ARCFLOW_QDRANT_URL` for real storage) |

### MemoryScope at a glance

| `MemoryScope` | Isolation |
|---------------|-----------|
| `AGENT` | Private to one agent id |
| `WORKFLOW` | Shared across agents in the same workflow run |
| `GLOBAL` | Shared across runs (use deliberately; most tutorials use `AGENT` or `WORKFLOW`) |

Combine type and scope on purpose. Example: `MemoryType.SHARED` with `MemoryScope.WORKFLOW` for pipeline handoff; `MemoryType.VECTOR` with `MemoryScope.AGENT` and a dedicated `namespace` for a single research agent.

Persistent and vector types require a non-empty `namespace`. Construction fails with `MemoryConfigurationError` if you omit it.

For local development on vector memory, set `embedding="stub/8"` (or `stub/384`). Stub embeddings are deterministic and need no API key. Production runs should swap to a real embedding string and configure `ARCFLOW_EMBEDDING_PROVIDER`.

## Example

Inspect the enums without running a workflow:

```python
from arcflow import MemoryConfig, MemoryScope, MemoryType

session = MemoryConfig(MemoryType.SESSION, MemoryScope.AGENT, namespace="scratch")
vector = MemoryConfig(
 MemoryType.VECTOR,
 MemoryScope.AGENT,
 namespace="doc_qa",
 embedding="stub/8",
)

print(session.memory_type, session.scope, session.namespace)
print(vector.memory_type, vector.embedding)
```

Attach memory to an agent (vector config matches [RAG chatbot walkthrough](../../examples/rag-chatbot.md) on the current branch):

```python
from arcflow import Agent, MemoryConfig, MemoryScope, MemoryType

agent = Agent(
 name="researcher",
 role="researcher",
 instructions="Answer using retrieved context.",
 memory=MemoryConfig(
 MemoryType.VECTOR,
 MemoryScope.AGENT,
 namespace="doc_qa",
 embedding="stub/8",
 ),
)
```

On the examples restructure branch the same wiring appears in `examples/rag/memory_guide_qa.py` with namespace `platform-docs-memory-guide`.

## Verify

| Check | Expected |
|-------|----------|
| `MemoryConfig(MemoryType.VECTOR, MemoryScope.AGENT)` without namespace | `MemoryConfigurationError` |
| `MemoryConfig(MemoryType.SESSION, MemoryScope.AGENT, namespace="scratch")` | Constructs; `namespace` is `"scratch"` |
| Vector config with `embedding="stub/8"` | Constructs; no env vars required for the enum exercise |

## Next

[02 Session and shared memory](02-session-and-shared-memory.md) covers in-run backends that do not need Qdrant or Postgres.
