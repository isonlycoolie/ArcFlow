"""Unit tests for arcflow.Agent."""

import pytest

from arcflow import Agent
from arcflow.exceptions import WorkflowConfigurationError


def test_agent_repr() -> None:
    agent = Agent(name="a", role="r", instructions="do work")
    assert "researcher" not in repr(agent)
    assert "a" in repr(agent)


def test_agent_rejects_empty_name() -> None:
    with pytest.raises(WorkflowConfigurationError, match=r"\[ArcFlow\]"):
        Agent(name="  ", role="r", instructions="i")


def test_binding_tuple_has_sprint4_fields() -> None:
    agent = Agent(name="n", role="role", instructions="inst")
    row = agent.binding_tuple()
    assert len(row) == 6
    assert row[1] == "n"
    assert row[4] == []
    assert row[5] is None
