"""Trading starter e2e in stub mode (Phase 2-Pro exit)."""

from __future__ import annotations

from arcflow import Agent, Workflow


def test_trading_starter_stub_e2e() -> None:
    researcher = Agent("researcher", "researcher", "Research.")
    analyst = Agent("analyst", "analyst", "Analyze.")
    strategist = Agent("strategist", "strategist", "Plan.")
    wf = (
        Workflow("paper_trade")
        .step(researcher)
        .step(analyst)
        .step(strategist)
    )
    results = wf.test([{"input": "Analyze AAPL swing trade", "expect_contains": "researcher"}])
    assert results[0]["passed"] is True
