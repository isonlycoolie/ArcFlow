"""Workflow execution result types."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True)
class WorkflowResult:
    """Output of a completed workflow run."""

    output: str
    run_id: str
    step_count: int
    trace_events: tuple[dict[str, Any], ...] = ()
    status: str = "completed"
    approval_key: str | None = None

    def __repr__(self) -> str:
        preview = self.output[:50]
        suffix = "..." if len(self.output) > 50 else ""
        return (
            f"WorkflowResult(run_id={self.run_id!r}, "
            f"step_count={self.step_count}, "
            f"output={preview!r}{suffix})"
        )
