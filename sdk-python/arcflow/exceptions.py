"""ArcFlow exception hierarchy."""

from __future__ import annotations


class ArcFlowError(Exception):
    """Base class for all ArcFlow exceptions."""


class WorkflowConfigurationError(ArcFlowError):
    """Raised when a workflow is mis-configured before execution."""


class WorkflowExecutionError(ArcFlowError):
    """Raised when a workflow step fails during execution."""

    def __init__(
        self,
        message: str,
        run_id: str | None = None,
        failed_step: str | None = None,
    ) -> None:
        super().__init__(message)
        self.run_id = run_id
        self.failed_step = failed_step

    def with_context(
        self,
        run_id: str | None,
        failed_step: str | None,
    ) -> WorkflowExecutionError:
        self.run_id = run_id
        self.failed_step = failed_step
        return self


class ToolConfigurationError(ArcFlowError):
    """Raised when a tool is mis-configured before execution."""


class ToolExecutionError(ArcFlowError):
    """Raised when a tool invocation fails during a workflow run."""

    def __init__(
        self,
        message: str,
        tool_name: str | None = None,
        run_id: str | None = None,
        failed_step: str | None = None,
    ) -> None:
        super().__init__(message)
        self.tool_name = tool_name
        self.run_id = run_id
        self.failed_step = failed_step


class MemoryConfigurationError(ArcFlowError):
    """Raised when memory configuration violates ArcFlow rules."""


class MemoryOperationError(ArcFlowError):
    """Raised when a memory read or write fails."""


class TraceNotFoundError(ArcFlowError):
    """No trace for the requested run / last run."""


class TraceStorageWarning(ArcFlowError):
    """Trace store dropped events for a run."""

    def __init__(self, message: str, events_dropped: int = 0, run_id: str | None = None) -> None:
        super().__init__(message)
        self.events_dropped = events_dropped
        self.run_id = run_id


class InfrastructureUnavailableError(ArcFlowError):
    """Raised when an optional memory backend is unreachable or unset."""

    def __init__(
        self,
        message: str,
        backend: str | None = None,
        suggestion: str | None = None,
    ) -> None:
        super().__init__(message)
        self.backend = backend
        self.suggestion = suggestion
