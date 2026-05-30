"""Human-in-the-loop configuration and errors."""

from __future__ import annotations

from dataclasses import dataclass

from arcflow.exceptions import ArcFlowError


@dataclass(frozen=True)
class HitlConfig:
    """Declarative human approval gate on a workflow step."""

    approval_key: str
    timeout_seconds: int = 3600
    interrupt: bool = True

    def to_json(self) -> str:
        import json

        return json.dumps(
            {
                "approval_key": self.approval_key,
                "timeout_seconds": self.timeout_seconds,
                "interrupt": self.interrupt,
            }
        )


class HumanRejectedError(ArcFlowError):
    """Raised when a human rejects an approval request."""

    def __init__(self, message: str, *, approval_key: str | None = None) -> None:
        super().__init__(message)
        self.approval_key = approval_key


class WorkflowInterruptedError(ArcFlowError):
    """Raised when a workflow pauses for human approval."""

    def __init__(
        self,
        message: str,
        *,
        run_id: str,
        approval_key: str,
        expires_at: str | None = None,
    ) -> None:
        super().__init__(message)
        self.run_id = run_id
        self.approval_key = approval_key
        self.expires_at = expires_at
