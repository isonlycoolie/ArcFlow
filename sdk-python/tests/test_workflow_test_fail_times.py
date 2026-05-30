"""Deterministic fail_times stub case."""

from __future__ import annotations

from arcflow import Agent, Workflow


def test_workflow_test_fail_times_then_output() -> None:
    agent = Agent(name="writer", role="author", instructions="Write.")
    wf = Workflow("fail_times_wf").step(agent)
    results = wf.test(
        [
            {
                "name": "recover after stub failures",
                "input": "hello",
                "expected_output": "recovered",
                "stub_responses": {
                    "step_1": {"fail_times": 2, "then_output": "recovered"},
                },
            }
        ]
    )
    assert len(results) == 1
    assert results[0]["passed"] is True
    assert results[0]["output"] == "recovered"
