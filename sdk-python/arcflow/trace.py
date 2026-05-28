"""Structured execution traces (Sprint 5). See contracts/ACD-005-python.md."""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime
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
