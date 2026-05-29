"""Bridge to the native binding module."""

from __future__ import annotations

import json
from typing import Any
from arcflow.agent import Agent
from arcflow.result import WorkflowResult
from arcflow.trace import TraceResult

try:
    from arcflow._arcflow_binding import WorkflowResult as NativeWorkflowResult
    from arcflow._arcflow_binding import (
        execute_resume_workflow,
        execute_workflow,
        get_execution_trace_json,
    )
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
    workflow_id: str,
    steps: list[Agent],
    step_rows: list[tuple[str, str, int, str | None]],
    run_input: str,
    provider: tuple[str, str, int, float] | None = None,
    exec_config_json: str | None = None,
) -> WorkflowResult:
    """Delegates execution to the Rust runtime via PyO3."""
    agent_rows = [agent.binding_tuple() for agent in steps]
    binding_steps = [
        (sid, aid, order, fb or "")
        for sid, aid, order, fb in step_rows
    ]
    tool_executors = [tool.execute for agent in steps for tool in agent.tools]
    native = execute_workflow(
        workflow_name,
        workflow_id,
        agent_rows,
        binding_steps,
        run_input,
        tool_executors,
        provider,
        exec_config_json,
    )
    return _to_result(native)


def resume_workflow(
    workflow_name: str,
    workflow_id: str,
    steps: list[Agent],
    step_rows: list[tuple[str, str, int, str | None]],
    original_run_id: str,
    exec_config_json: str | None = None,
) -> WorkflowResult:
    """Resumes a failed workflow from PostgreSQL recovery state."""
    agent_rows = [agent.binding_tuple() for agent in steps]
    binding_steps = [
        (sid, aid, order, fb or "")
        for sid, aid, order, fb in step_rows
    ]
    tool_executors = [tool.execute for agent in steps for tool in agent.tools]
    native = execute_resume_workflow(
        workflow_name,
        workflow_id,
        agent_rows,
        binding_steps,
        original_run_id,
        tool_executors,
        None,
        exec_config_json,
    )
    return _to_result(native)


def get_trace(run_id: str) -> TraceResult:
    """Loads a structured trace from the in-process Rust store."""
    return TraceResult.from_json(get_execution_trace_json(run_id))
