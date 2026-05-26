"""Canonical first-five-minutes integration test."""

from arcflow import Agent, Workflow


def test_canonical_minimal_example() -> None:
    researcher = Agent(
        name="researcher",
        role="research",
        instructions="Research deeply.",
    )
    writer = Agent(
        name="writer",
        role="write",
        instructions="Write clearly.",
    )

    workflow = Workflow()
    workflow.step(researcher)
    workflow.step(writer)

    result = workflow.run("Analyze renewable energy trends")

    assert result is not None
    assert len(result.output) > 0
    assert result.step_count == 2
    assert result.run_id
