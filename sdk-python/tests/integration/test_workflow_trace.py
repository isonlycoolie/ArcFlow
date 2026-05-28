"""Sprint 5 workflow.trace() integration tests."""

from __future__ import annotations

from arcflow import Agent, TraceNotFoundError, Workflow
from arcflow.trace import TraceResult


def test_trace_after_run_returns_steps() -> None:
    wf = Workflow("trace-wf")
    wf.step(Agent(name="a", role="researcher", instructions="work"))
    wf.step(Agent(name="b", role="coder", instructions="work"))
    result = wf.run("hello trace")
    trace = wf.trace()
    assert isinstance(trace, TraceResult)
    assert trace.run_id == result.run_id
    assert len(trace) == 2
    assert trace.status in ("completed", "partial")
    assert trace.total_duration_seconds >= 0


def test_trace_before_run_raises() -> None:
    wf = Workflow("empty-trace")
    wf.step(Agent(name="a", role="researcher", instructions="work"))
    try:
        wf.trace()
        raise AssertionError("expected TraceNotFoundError")
    except TraceNotFoundError:
        pass
