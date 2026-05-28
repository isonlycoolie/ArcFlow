"""Bridge to the native binding module."""

from __future__ import annotations

import json
from typing import Any
from uuid import uuid4

from arcflow.agent import Agent
from arcflow.result import WorkflowResult
from arcflow.trace import TraceResult

try:
    from arcflow._arcflow_binding import WorkflowResult as NativeWorkflowResult
    from arcflow._arcflow_binding import execute_workflow, get_execution_trace_json
except ImportError as exc:  # pragma: no cover - built by maturin
    raise ImportError(
        "[ArcFlow] Native extension not installed. "
        "Install with: pip install maturin && maturin develop"
    ) from exc


def _parse_trace_events(raw: str) -> tuple[dict[str, Any], ...]:
    if not raw or raw == "[]":
        return ()
    parsed = json.loads(raw)
    if not isinstance(parsed, list):
        return ()
    return tuple(item for item in parsed if isinstance(item, dict))


def _to_result(native: NativeWorkflowResult) -> WorkflowResult:
    return WorkflowResult(
        output=native.output,
        run_id=native.run_id,
        step_count=native.step_count,
        trace_events=_parse_trace_events(native.trace_events_json),
    )


def run_workflow(
    workflow_name: str,
    steps: list[Agent],
    run_input: str,
    provider: tuple[str, str, int, float] | None = None,
) -> WorkflowResult:
    """Delegates execution to the Rust runtime via PyO3."""
    agent_rows = [agent.binding_tuple() for agent in steps]
    step_rows: list[tuple[str, str, int]] = []
    for index, agent in enumerate(steps, start=1):
        step_rows.append((str(uuid4()), str(agent.agent_id), index))
    tool_executors = [tool.execute for agent in steps for tool in agent.tools]
    native = execute_workflow(
        workflow_name,
        str(uuid4()),
        agent_rows,
        step_rows,
        run_input,
        tool_executors,
        provider,
    )
    return _to_result(native)


def get_trace(run_id: str) -> TraceResult:
    """Loads a structured trace from the in-process Rust store."""
    return TraceResult.from_json(get_execution_trace_json(run_id))
