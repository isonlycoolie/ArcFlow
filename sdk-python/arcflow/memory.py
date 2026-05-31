"""Memory configuration for agents."""

from __future__ import annotations

from enum import Enum

from arcflow.exceptions import MemoryConfigurationError


class MemoryType(str, Enum):
    """Memory backend kind (matches RCS)."""

    SESSION = "Session"
    SHARED = "Shared"
    PERSISTENT = "Persistent"
    VECTOR = "Vector"


class MemoryScope(str, Enum):
    """Memory access scope (matches RCS)."""

    AGENT = "Agent"
    WORKFLOW = "Workflow"
    GLOBAL = "Global"


class MemoryRetrievalConfig:
    """Hybrid retrieval settings for vector memory (Phase 2.5)."""

    def __init__(
        self,
        mode: str = "dense",
        dense_weight: float = 0.7,
        sparse_weight: float = 0.3,
        rerank: str | None = None,
        top_k: int | None = None,
    ) -> None:
        if mode not in ("dense", "hybrid"):
            raise MemoryConfigurationError(
                "[ArcFlow] retrieval.mode must be 'dense' or 'hybrid'."
            )
        if rerank is not None and rerank not in ("cohere", "local"):
            raise MemoryConfigurationError(
                "[ArcFlow] retrieval.rerank must be 'cohere', 'local', or omitted."
            )
        self.mode = mode
        self.dense_weight = dense_weight
        self.sparse_weight = sparse_weight
        self.rerank = rerank
        self.top_k = top_k


class MemoryChunkingConfig:
    """Document chunking settings for vector ingest (Phase 2.5)."""

    def __init__(
        self,
        strategy: str = "recursive",
        chunk_size: int = 512,
        overlap: int = 64,
    ) -> None:
        if chunk_size < 64:
            raise MemoryConfigurationError(
                "[ArcFlow] chunking.chunk_size must be at least 64."
            )
        if overlap < 0:
            raise MemoryConfigurationError(
                "[ArcFlow] chunking.overlap must be non-negative."
            )
        self.strategy = strategy
        self.chunk_size = chunk_size
        self.overlap = overlap


class MemoryConfig:
    """Configures how an agent uses memory during a workflow run."""

    namespace: str | None

    def __init__(
        self,
        memory_type: MemoryType,
        scope: MemoryScope = MemoryScope.AGENT,
        namespace: str | None = None,
        ttl_seconds: int | None = None,
        embedding: str | None = None,
        retrieval: MemoryRetrievalConfig | None = None,
        chunking: MemoryChunkingConfig | None = None,
    ) -> None:
        if not isinstance(memory_type, MemoryType):
            raise MemoryConfigurationError(
                "[ArcFlow] memory_type must be a MemoryType enum value."
            )
        if not isinstance(scope, MemoryScope):
            raise MemoryConfigurationError(
                "[ArcFlow] scope must be a MemoryScope enum value."
            )
        if memory_type in (MemoryType.PERSISTENT, MemoryType.VECTOR):
            ns = (namespace or "").strip()
            if not ns:
                raise MemoryConfigurationError(
                    "[ArcFlow] namespace is required for persistent and vector memory."
                )
            self.namespace = ns
        else:
            self.namespace = namespace.strip() if namespace else None
        if ttl_seconds is not None and ttl_seconds <= 0:
            raise MemoryConfigurationError(
                "[ArcFlow] ttl_seconds must be positive when set."
            )
        self.memory_type = memory_type
        self.scope = scope
        self.ttl_seconds = ttl_seconds
        self.embedding = embedding.strip() if embedding else None
        self.retrieval = retrieval
        self.chunking = chunking

    def binding_json(self) -> str | None:
        """JSON payload for the native binding, or None when unset."""
        import json

        payload: dict[str, object] = {
            "memory_type": self.memory_type.value,
            "scope": self.scope.value,
            "namespace": self.namespace,
            "ttl_seconds": self.ttl_seconds,
        }
        if self.embedding:
            payload["embedding"] = self.embedding
        if self.retrieval is not None:
            payload["retrieval"] = {
                "mode": self.retrieval.mode,
                "dense_weight": self.retrieval.dense_weight,
                "sparse_weight": self.retrieval.sparse_weight,
                "rerank": self.retrieval.rerank,
                "top_k": self.retrieval.top_k,
            }
        if self.chunking is not None:
            payload["chunking"] = {
                "strategy": self.chunking.strategy,
                "chunk_size": self.chunking.chunk_size,
                "overlap": self.chunking.overlap,
            }
        return json.dumps(payload)


class ChunkHit:
    """One retrieved vector chunk."""

    def __init__(self, text: str, byte_len: int) -> None:
        self.text = text
        self.byte_len = byte_len


class VectorStore:
    """SDK binding for vector ingest and search (Phase 2-Pro)."""

    def __init__(self) -> None:
        from arcflow._arcflow_binding import PyVectorStore

        self._native = PyVectorStore()

    def ingest(self, namespace: str, key: str, text: str) -> int:
        ns = (namespace or "").strip()
        if not ns:
            raise MemoryConfigurationError("[ArcFlow] VectorStore.ingest requires namespace.")
        return int(self._native.ingest(ns, key, text))

    def search(self, namespace: str, query: str, top_k: int = 5) -> list[ChunkHit]:
        ns = (namespace or "").strip()
        if not ns:
            raise MemoryConfigurationError("[ArcFlow] VectorStore.search requires namespace.")
        hits = self._native.search_hits(ns, query, top_k)
        return [ChunkHit(text=t, byte_len=n) for t, n in hits]

