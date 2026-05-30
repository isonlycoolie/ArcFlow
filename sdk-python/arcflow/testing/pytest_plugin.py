"""Pytest fixtures for ArcFlow workflow tests."""

from __future__ import annotations

from typing import Any

import pytest


def pytest_configure(config: pytest.Config) -> None:
    config.addinivalue_line(
        "markers",
        "arcflow_stub_responses(step_1=dict): inject stub responses for workflow.test()",
    )


@pytest.fixture
def arcflow_stub_responses(request: pytest.FixtureRequest) -> dict[str, Any]:
    """Stub responses from ``@pytest.mark.arcflow_stub_responses(step_1={...})``."""
    marker = request.node.get_closest_marker("arcflow_stub_responses")
    if marker is None:
        return {}
    if marker.args and isinstance(marker.args[0], dict):
        return marker.args[0]
    return dict(marker.kwargs)
