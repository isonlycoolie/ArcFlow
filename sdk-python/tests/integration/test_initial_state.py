"""Python SDK initial_state integration (Phase 2-Pro)."""

from __future__ import annotations

from arcflow import Agent, Workflow


def test_workflow_run_accepts_initial_state() -> None:
    agent = Agent("observer", "observer", "Observe.")
    wf = Workflow("initial_state_smoke", graph=True).node("observe", agent, outputs=["observation"])
    wf.add_edge("observe", None)
    results = wf.test(
        [{"input": "task", "expect_contains": "observer"}],
    )
    assert results[0]["passed"] is True

    wf2 = Workflow("initial_state_run").step(agent)
    result = wf2.run("task", initial_state={"seed": "context"})
    assert result.output
