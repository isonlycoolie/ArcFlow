"""Type stubs for the native extension module."""

from typing import Any, Callable

class WorkflowResult:
    output: str
    run_id: str
    step_count: int

def execute_workflow(
    workflow_name: str,
    workflow_id: str,
    agents: list[
        tuple[str, str, str, str, list[tuple[str, str, str, float]], str | None]
    ],
    steps: list[tuple[str, str, int]],
    run_input: str,
    tool_executors: list[Callable[..., Any]],
) -> WorkflowResult: ...
