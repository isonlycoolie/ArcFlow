"""Trace lookup performance (Sprint 5 DoD)."""

from __future__ import annotations

import time

from arcflow import Agent, Workflow


def test_trace_lookup_under_10ms_for_100_step_workflow() -> None:
    wf = Workflow("perf-trace")
    for index in range(100):
        wf.step(Agent(name=f"agent-{index}", role="researcher", instructions="work"))
    wf.run("perf-input")
    wf.trace()  # warm cache / one-time JSON parse
    samples = [
        _trace_lookup_seconds(wf) for _ in range(5)
    ]
    elapsed = min(samples)
    assert elapsed < 0.010, f"trace lookup took {elapsed * 1000:.2f}ms (min of {len(samples)})"


def _trace_lookup_seconds(wf: Workflow) -> float:
    start = time.perf_counter()
    wf.trace()
    return time.perf_counter() - start
