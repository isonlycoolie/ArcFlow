"""User-facing stream events from workflow.run_stream() (Phase 2.1)."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from arcflow.exceptions import WorkflowExecutionError


@dataclass(frozen=True)
class StreamEvent:
    """One event emitted while a workflow runs with streaming enabled."""

    type: str
    text: str | None = None
    step_id: str | None = None
    node_id: str | None = None
    duration_ms: int | None = None
    tool_name: str | None = None
    args_keys: tuple[str, ...] | None = None
    code: str | None = None
    message: str | None = None

    @classmethod
    def from_dict(cls, raw: dict[str, Any]) -> StreamEvent:
        event_type = str(raw.get("type", ""))
        args_keys = raw.get("args_keys")
        keys: tuple[str, ...] | None = None
        if isinstance(args_keys, list):
            keys = tuple(str(k) for k in args_keys)
        return cls(
            type=event_type,
            text=raw.get("text") if raw.get("text") is not None else None,
            step_id=raw.get("step_id") if raw.get("step_id") is not None else None,
            node_id=raw.get("node_id") if raw.get("node_id") is not None else None,
            duration_ms=raw.get("duration_ms"),
            tool_name=raw.get("tool_name"),
            args_keys=keys,
            code=raw.get("code"),
            message=raw.get("message"),
        )

    def to_exception(self) -> WorkflowExecutionError:
        if self.type != "error":
            raise ValueError(
                f"StreamEvent.to_exception() requires type 'error', got {self.type!r}."
            )
        return WorkflowExecutionError(
            self.message or "[ArcFlow] Stream error event.",
            failed_step=self.step_id,
        )


@dataclass(frozen=True)
class StreamRunResult:
    """Final workflow result after a streamed run completes."""

    output: str
    run_id: str
    step_count: int
