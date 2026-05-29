"""Structured execution traces. See contracts/TRACE-EVENT-SCHEMA-v1.md."""

from __future__ import annotations

import json
from collections.abc import Iterator
from dataclasses import dataclass
from datetime import datetime, timezone


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
        parsed = json.loads(raw)
        if not isinstance(parsed, dict):
            raise ValueError("trace JSON must be an object")
        data: dict[str, object] = parsed
        steps = tuple(_parse_step(s) for s in _as_dict_list(data.get("steps", [])))
        tokens = _as_dict(data.get("total_tokens", {}))
        dropped = _as_int(data.get("events_dropped", 0))
        warnings: list[str] = []
        if dropped:
            warnings.append(f"{dropped} trace events dropped (store capacity)")
        return cls(
            run_id=str(data["run_id"]),
            workflow_name=str(data.get("workflow_name", "unknown")),
            status=_status_str(data.get("status")),
            started_at=_parse_ts(data["started_at"]),
            completed_at=(
                _parse_ts(data["completed_at"]) if data.get("completed_at") else None
            ),
            total_duration_seconds=_ms_to_seconds(data.get("duration_ms")),
            total_tokens_consumed=_as_int(tokens.get("total_tokens", 0)),
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


def _as_dict(value: object) -> dict[str, object]:
    if isinstance(value, dict):
        return value
    return {}


def _as_dict_list(value: object) -> list[dict[str, object]]:
    if not isinstance(value, list):
        return []
    return [item for item in value if isinstance(item, dict)]


def _as_int(value: object, default: int = 0) -> int:
    if isinstance(value, int):
        return value
    return default


def _ms_to_seconds(value: object) -> float:
    if isinstance(value, int):
        return value / 1000.0
    return 0.0


def _parse_step(raw: dict[str, object]) -> StepTrace:
    tools = tuple(_parse_tool(t) for t in _as_dict_list(raw.get("tool_calls", [])))
    mem_ops = tuple(
        _parse_memory(m) for m in _as_dict_list(raw.get("memory_operations", []))
    )
    err_raw = raw.get("error")
    err = None
    if isinstance(err_raw, dict):
        err = StepError(
            error_code=str(err_raw.get("error_code", "")),
            message=str(err_raw.get("message", "")),
        )
    tokens = _as_dict(raw.get("tokens", {}))
    step_status = str(raw.get("status", "Completed"))
    if "InProgress" in step_status:
        step_status = "completed"
    else:
        step_status = step_status.lower().replace("inprogress", "completed")
    return StepTrace(
        step_index=_as_int(raw.get("step_index", 0)),
        agent_name=str(raw.get("agent_name", "")),
        agent_role=str(raw.get("agent_role", "")),
        status=step_status if step_status in ("completed", "failed") else "completed",
        started_at=_parse_ts(raw["started_at"]),
        completed_at=(
            _parse_ts(raw["completed_at"]) if raw.get("completed_at") else None
        ),
        duration_seconds=_ms_to_seconds(raw.get("duration_ms")),
        tokens_consumed=TokenUsage(
            _as_int(tokens.get("prompt_tokens", 0)),
            _as_int(tokens.get("completion_tokens", 0)),
            _as_int(tokens.get("total_tokens", 0)),
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
    output_size = raw.get("output_size_bytes")
    error_code = raw.get("error_code")
    return ToolCallTrace(
        tool_name=str(raw.get("tool_name", "")),
        status=status,
        duration_seconds=_ms_to_seconds(raw.get("duration_ms")),
        input_schema_hash=str(raw.get("input_schema_hash", "")),
        output_size_bytes=output_size if isinstance(output_size, int) else None,
        error_code=str(error_code) if isinstance(error_code, str) else None,
    )


def _parse_memory(raw: dict[str, object]) -> MemoryOperationTrace:
    op = str(raw.get("operation", "Read"))
    hit_value = raw.get("hit")
    return MemoryOperationTrace(
        operation="read" if "Read" in op else "write",
        memory_type=str(raw.get("memory_type", "")),
        key=str(raw.get("key", "")),
        hit=hit_value if isinstance(hit_value, bool) else None,
        duration_seconds=_ms_to_seconds(raw.get("duration_ms")),
    )
