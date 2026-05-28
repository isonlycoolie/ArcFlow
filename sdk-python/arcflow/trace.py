"""Structured execution traces (Sprint 5). See contracts/ACD-005-python.md."""

from __future__ import annotations

import json
from dataclasses import dataclass
from datetime import datetime, timezone
from typing import Iterator


@dataclass(frozen=True)
class TokenUsage:
    prompt_tokens: int
    completion_tokens: int
    total_tokens: int


@dataclass(frozen=True)
class ToolCallTrace:
    tool_name: str
    status: str
    duration_seconds: float
    input_schema_hash: str
    output_size_bytes: int | None
    error_code: str | None


@dataclass(frozen=True)
class MemoryOperationTrace:
    operation: str
    memory_type: str
    key: str
    hit: bool | None
    duration_seconds: float


@dataclass(frozen=True)
class StepError:
    error_code: str
    message: str


@dataclass(frozen=True)
class StepTrace:
    step_index: int
    agent_name: str
    agent_role: str
    status: str
    started_at: datetime
    completed_at: datetime | None
    duration_seconds: float
    tokens_consumed: TokenUsage
    tools_called: tuple[ToolCallTrace, ...]
    memory_operations: tuple[MemoryOperationTrace, ...]
    error: StepError | None


@dataclass(frozen=True)
class TraceResult:
    run_id: str
    workflow_name: str
    status: str
    started_at: datetime
    completed_at: datetime | None
    total_duration_seconds: float
    total_tokens_consumed: int
    steps: tuple[StepTrace, ...]
    warnings: tuple[str, ...]

    def summary(self) -> str:
        return (
            f"{self.workflow_name} ({self.status}): "
            f"{len(self.steps)} steps, {self.total_tokens_consumed} tokens"
        )

    def failed_step(self) -> StepTrace | None:
        for step in self.steps:
            if step.status == "failed":
                return step
        return None

    def __iter__(self) -> Iterator[StepTrace]:
        return iter(self.steps)

    def __len__(self) -> int:
        return len(self.steps)

    @classmethod
    def from_json(cls, raw: str) -> TraceResult:
        data = json.loads(raw)
        steps = tuple(_parse_step(s) for s in data.get("steps", []))
        tokens = data.get("total_tokens", {})
        dropped = int(data.get("events_dropped", 0))
        warnings: list[str] = []
        if dropped:
            warnings.append(f"{dropped} trace events dropped (store capacity)")
        duration_ms = data.get("duration_ms")
        return cls(
            run_id=str(data["run_id"]),
            workflow_name=str(data.get("workflow_name", "unknown")),
            status=_status_str(data.get("status")),
            started_at=_parse_ts(data["started_at"]),
            completed_at=_parse_ts(data["completed_at"])
            if data.get("completed_at")
            else None,
            total_duration_seconds=(duration_ms or 0) / 1000.0,
            total_tokens_consumed=int(tokens.get("total_tokens", 0)),
            steps=steps,
            warnings=tuple(warnings),
        )


def _status_str(value: object) -> str:
    if isinstance(value, str):
        if value.startswith("{"):
            return "partial"
        return value.lower()
    return "partial"


def _parse_ts(value: object) -> datetime:
    text = str(value).replace("Z", "+00:00")
    return datetime.fromisoformat(text).astimezone(timezone.utc)


def _parse_step(raw: dict[str, object]) -> StepTrace:
    tools = tuple(_parse_tool(t) for t in raw.get("tool_calls", []) if isinstance(t, dict))
    mem_ops = tuple(
        _parse_memory(m) for m in raw.get("memory_operations", []) if isinstance(m, dict)
    )
    err_raw = raw.get("error")
    err = None
    if isinstance(err_raw, dict):
        err = StepError(
            error_code=str(err_raw.get("error_code", "")),
            message=str(err_raw.get("message", "")),
        )
    tokens = raw.get("tokens", {})
    step_status = str(raw.get("status", "Completed"))
    if "InProgress" in step_status:
        step_status = "completed"
    else:
        step_status = step_status.lower().replace("inprogress", "completed")
    duration_ms = raw.get("duration_ms")
    return StepTrace(
        step_index=int(raw.get("step_index", 0)),
        agent_name=str(raw.get("agent_name", "")),
        agent_role=str(raw.get("agent_role", "")),
        status=step_status if step_status in ("completed", "failed") else "completed",
        started_at=_parse_ts(raw["started_at"]),
        completed_at=_parse_ts(raw["completed_at"])
        if raw.get("completed_at")
        else None,
        duration_seconds=(duration_ms or 0) / 1000.0,
        tokens_consumed=TokenUsage(
            int(tokens.get("prompt_tokens", 0)),
            int(tokens.get("completion_tokens", 0)),
            int(tokens.get("total_tokens", 0)),
        ),
        tools_called=tools,
        memory_operations=mem_ops,
        error=err,
    )


def _parse_tool(raw: dict[str, object]) -> ToolCallTrace:
    status = str(raw.get("status", "Success"))
    if "Success" in status:
        status = "success"
    else:
        status = status.lower()
    return ToolCallTrace(
        tool_name=str(raw.get("tool_name", "")),
        status=status,
        duration_seconds=int(raw.get("duration_ms", 0)) / 1000.0,
        input_schema_hash=str(raw.get("input_schema_hash", "")),
        output_size_bytes=raw.get("output_size_bytes"),
        error_code=raw.get("error_code"),  # type: ignore[arg-type]
    )


def _parse_memory(raw: dict[str, object]) -> MemoryOperationTrace:
    op = str(raw.get("operation", "Read"))
    return MemoryOperationTrace(
        operation="read" if "Read" in op else "write",
        memory_type=str(raw.get("memory_type", "")),
        key=str(raw.get("key", "")),
        hit=raw.get("hit"),  # type: ignore[arg-type]
        duration_seconds=int(raw.get("duration_ms", 0)) / 1000.0,
    )
