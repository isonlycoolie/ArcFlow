"""workflow.test() deterministic stub cases."""

from __future__ import annotations

from arcflow import Agent, Workflow


def test_workflow_test_happy_path() -> None:
    agent = Agent(name="writer", role="author", instructions="Write.")
    wf = Workflow("test_wf").step(agent)
    results = wf.test(
        [
            {
                "name": "happy path",
                "input": "hello",
                "expected_output": "fixed",
                "stub_responses": {"step_1": {"output": "fixed"}},
            }
        ]
    )
    assert len(results) == 1
    assert results[0]["passed"] is True
