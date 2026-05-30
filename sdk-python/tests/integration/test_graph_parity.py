"""Graph workflow integration — cross-fixture parity."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from arcflow import Agent, Workflow

FIXTURE = (
    Path(__file__).resolve().parents[3] / "tests" / "fixtures" / "linear_graph.rcs.json"
)


def _agent(name: str) -> Agent:
    return Agent(name=name, role=name, instructions=f"Run {name}.")


@pytest.fixture
def linear_graph_fixture() -> dict:
    return json.loads(FIXTURE.read_text(encoding="utf-8"))


def test_linear_graph_step_count_matches_fixture(
    linear_graph_fixture: dict,
) -> None:
    wf = Workflow("linear_graph_parity", graph=True)
    nodes = {n["id"]: n for n in linear_graph_fixture["graph"]["nodes"]}
    wf.node("first", _agent("first"))
    wf.node("second", _agent("second"))
    wf.set_entry("first")
    wf.add_edge("first", "second")
    wf.add_edge("second", None)
    result = wf.run("parity-input")
    assert result.step_count == len(linear_graph_fixture["steps"])


REACT_FIXTURE = (
    Path(__file__).resolve().parents[3] / "tests" / "fixtures" / "react_graph.rcs.json"
)


@pytest.fixture
def react_graph_fixture() -> dict:
    return json.loads(REACT_FIXTURE.read_text(encoding="utf-8"))


def _build_react_workflow(max_iterations: int) -> Workflow:
    wf = Workflow("react_graph_parity", graph=True).max_iterations(max_iterations)
    wf.node("think", _agent("think"))
    wf.node("act", _agent("act"))
    wf.node("observe", _agent("observe"))
    wf.set_entry("think")
    wf.add_edge("think", "act")
    wf.add_edge("act", "observe")
    wf.add_edge("observe", "think")
    return wf


def test_react_graph_one_iteration_matches_fixture(
    react_graph_fixture: dict,
) -> None:
    result = _build_react_workflow(1).run("react-input")
    assert result.step_count == len(react_graph_fixture["steps"])
