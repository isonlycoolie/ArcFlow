"""Unit tests for graph-mode Workflow."""

import pytest

from arcflow import Agent, Workflow
from arcflow.exceptions import WorkflowConfigurationError


def _agent(name: str) -> Agent:
    return Agent(name=name, role=name, instructions=f"Run {name}.")


def test_graph_node_forbids_step() -> None:
    wf = Workflow("g", graph=True)
    wf.node("a", _agent("a"))
    with pytest.raises(WorkflowConfigurationError, match=r"step\(\)"):
        wf.step(_agent("b"))


def test_linear_forbids_node() -> None:
    wf = Workflow("linear")
    with pytest.raises(WorkflowConfigurationError, match=r"graph=True"):
        wf.node("a", _agent("a"))


def test_graph_run_requires_nodes() -> None:
    with pytest.raises(WorkflowConfigurationError, match=r"no nodes"):
        Workflow("g", graph=True).run("input")
