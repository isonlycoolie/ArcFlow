"""Pytest plugin registration for ArcFlow."""

from __future__ import annotations

import pytest


pytest_plugins = ["arcflow.testing.pytest_plugin"]


def test_arcflow_stub_responses_fixture(arcflow_stub_responses: dict[str, object]) -> None:
    assert isinstance(arcflow_stub_responses, dict)


@pytest.mark.arcflow
def test_arcflow_marker_registered(arcflow_workflow) -> None:
    assert arcflow_workflow is not None
