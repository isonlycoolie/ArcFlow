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


class MemoryConfig:
    """Configures how an agent uses memory during a workflow run."""

    namespace: str | None

    def __init__(
        self,
        memory_type: MemoryType,
        scope: MemoryScope = MemoryScope.AGENT,
        namespace: str | None = None,
        ttl_seconds: int | None = None,
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

    def binding_json(self) -> str | None:
        """JSON payload for the native binding, or None when unset."""
        import json

        return json.dumps(
            {
                "memory_type": self.memory_type.value,
                "scope": self.scope.value,
                "namespace": self.namespace,
                "ttl_seconds": self.ttl_seconds,
            }
        )
