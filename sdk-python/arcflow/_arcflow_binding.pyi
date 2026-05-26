"""Type stubs for the native extension module."""

class WorkflowResult:
    output: str
    run_id: str
    step_count: int

def execute_workflow(
    workflow_name: str,
    workflow_id: str,
    agents: list[tuple[str, str, str, str]],
    steps: list[tuple[str, str, int]],
    run_input: str,
) -> WorkflowResult: ...
