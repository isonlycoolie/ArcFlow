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
