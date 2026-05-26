"""Unit tests for arcflow.exceptions."""

from arcflow.exceptions import (
    ArcFlowError,
    WorkflowConfigurationError,
    WorkflowExecutionError,
)


def test_workflow_configuration_error_is_arcflow_error() -> None:
    err = WorkflowConfigurationError("[ArcFlow] test.")
    assert isinstance(err, ArcFlowError)


def test_workflow_execution_error_preserves_context() -> None:
    err = WorkflowExecutionError(
        "[ArcFlow] step failed.",
        run_id="abc-123",
        failed_step="writer",
    )
    assert err.run_id == "abc-123"
    assert err.failed_step == "writer"
    assert isinstance(err, ArcFlowError)


def test_error_message_format() -> None:
    msg = (
        "[ArcFlow] No steps defined. "
        "Add at least one step with workflow.step(agent)."
    )
    err = WorkflowConfigurationError(msg)
    assert str(err).startswith("[ArcFlow]")
