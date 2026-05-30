"""Pytest fixtures for ArcFlow workflow tests."""

from __future__ import annotations

from typing import Any

import pytest

from arcflow import Agent, Workflow


def pytest_configure(config: pytest.Config) -> None:
    config.addinivalue_line(
        "markers",
        "arcflow: ArcFlow workflow integration test (stub mode, no real LLM)",
    )
    config.addinivalue_line(
        "markers",
        "arcflow_stub_responses(step_1=dict): inject stub responses for workflow.test()",
    )


@pytest.fixture
def arcflow_workflow() -> Workflow:
    """Linear one-step workflow for deterministic stub tests."""
    agent = Agent(name="writer", role="author", instructions="Write briefly.")
    return Workflow("pytest_wf").step(agent)


@pytest.fixture
def arcflow_stub_responses(request: pytest.FixtureRequest) -> dict[str, Any]:
    """Stub responses from ``@pytest.mark.arcflow_stub_responses(step_1={...})``."""
    marker = request.node.get_closest_marker("arcflow_stub_responses")
    if marker is None:
        return {}
    if marker.args and isinstance(marker.args[0], dict):
        return marker.args[0]
    return dict(marker.kwargs)
