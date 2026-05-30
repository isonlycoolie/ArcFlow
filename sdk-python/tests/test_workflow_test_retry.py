"""assert_retries and pytest arcflow fixture tests."""

from __future__ import annotations

import pytest

from arcflow import Agent, Workflow

pytest_plugins = ["arcflow.testing.pytest_plugin"]


def test_assert_retries_case() -> None:
    agent = Agent(name="writer", role="author", instructions="Write.")
    wf = Workflow("retry_assert_wf").step(agent)
    results = wf.test(
        [
            {
                "name": "retry on failure",
                "input": "hello",
                "expected_output": "recovered",
                "mock_step_failure": "step_1",
                "mock_fail_count": 2,
                "assert_retries": 3,
            }
        ]
    )
    assert results[0]["passed"] is True
    assert results[0]["attempts_made"] == 3


@pytest.mark.arcflow
@pytest.mark.arcflow_stub_responses(step_1={"output": "from-marker"})
def test_arcflow_workflow_fixture(
    arcflow_workflow: Workflow, arcflow_stub_responses: dict[str, object]
) -> None:
    results = arcflow_workflow.test(
        [{"name": "marker case", "input": "hi", "stub_responses": arcflow_stub_responses}]
    )
    assert results[0]["passed"] is True
    assert results[0]["output"] == "from-marker"


def test_test_mode_recovery_disabled(monkeypatch: pytest.MonkeyPatch) -> None:
    """workflow.test() must not require Postgres (recovery off)."""
    monkeypatch.delenv("ARCFLOW_POSTGRESQL_URL", raising=False)
    agent = Agent(name="writer", role="author", instructions="Write.")
    wf = Workflow("no_pg_wf").step(agent)
    results = wf.test(
        [
            {
                "input": "hello",
                "expected_output": "ok",
                "stub_responses": {"step_1": {"output": "ok"}},
            }
        ]
    )
    assert results[0]["passed"] is True
