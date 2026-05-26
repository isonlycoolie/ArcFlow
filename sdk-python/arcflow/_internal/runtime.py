"""Bridge to the native binding module."""

from __future__ import annotations

from uuid import uuid4

from arcflow.agent import Agent
from arcflow.result import WorkflowResult

try:
    from arcflow._arcflow_binding import WorkflowResult as NativeWorkflowResult
    from arcflow._arcflow_binding import execute_workflow
except ImportError as exc:  # pragma: no cover - built by maturin
    raise ImportError(
        "[ArcFlow] Native extension not installed. "
        "Install with: pip install maturin && maturin develop"
    ) from exc


def _to_result(native: NativeWorkflowResult) -> WorkflowResult:
    return WorkflowResult(
        output=native.output,
        run_id=native.run_id,
        step_count=native.step_count,
    )


def run_workflow(
    workflow_name: str,
    steps: list[Agent],
    run_input: str,
) -> WorkflowResult:
    """Delegates execution to the Rust runtime via PyO3."""
    agent_rows = [agent.binding_tuple() for agent in steps]
    step_rows: list[tuple[str, str, int]] = []
    for index, agent in enumerate(steps, start=1):
        step_rows.append((str(uuid4()), str(agent.agent_id), index))
    native = execute_workflow(
        workflow_name,
        str(uuid4()),
        agent_rows,
        step_rows,
        run_input,
    )
    return _to_result(native)
