"""Unit tests for arcflow.Workflow (configuration only)."""

import pytest

from arcflow import Agent, Workflow
from arcflow.exceptions import WorkflowConfigurationError


@pytest.fixture
def researcher() -> Agent:
    return Agent(name="researcher", role="research", instructions="Research.")


def test_workflow_rejects_empty_name() -> None:
    with pytest.raises(WorkflowConfigurationError, match=r"\[ArcFlow\]"):
        Workflow(name="   ")


def test_workflow_run_requires_steps() -> None:
    with pytest.raises(WorkflowConfigurationError, match=r"no steps"):
        Workflow().run("input")


def test_workflow_run_rejects_empty_input(researcher: Agent) -> None:
    workflow = Workflow()
    workflow.step(researcher)
    with pytest.raises(WorkflowConfigurationError, match=r"\[ArcFlow\]"):
        workflow.run("")


def test_workflow_step_rejects_non_agent() -> None:
    workflow = Workflow()
    with pytest.raises(WorkflowConfigurationError, match=r"\[ArcFlow\]"):
        workflow.step("not an agent")  # type: ignore[arg-type]


def test_workflow_method_chaining(researcher: Agent) -> None:
    writer = Agent(name="writer", role="write", instructions="Write.")
    workflow = Workflow()
    returned = workflow.step(researcher).step(writer)
    assert returned is workflow
