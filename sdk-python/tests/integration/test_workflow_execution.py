"""Integration tests for multi-step workflow execution."""

import pytest

from arcflow import Agent, Workflow
from arcflow.exceptions import WorkflowConfigurationError, WorkflowExecutionError


def test_two_step_workflow_returns_result() -> None:
    a1 = Agent(name="a1", role="A", instructions="first")
    a2 = Agent(name="a2", role="B", instructions="second")
    result = Workflow().step(a1).step(a2).run("hello")
    assert result.step_count == 2
    assert "hello" in result.output or result.output


def test_workflow_run_with_no_steps_raises() -> None:
    with pytest.raises(WorkflowConfigurationError, match=r"no steps"):
        Workflow().run("input")


def test_workflow_halts_on_fail_role() -> None:
    ok = Agent(name="ok", role="ok", instructions="fine")
    fail = Agent(name="fail", role="__fail__", instructions="fail")
    workflow = Workflow().step(ok).step(fail)
    with pytest.raises(WorkflowExecutionError, match=r"\[ArcFlow\]"):
        workflow.run("trigger failure")
