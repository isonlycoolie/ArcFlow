"""Ensure traces never expose sensitive payload values (Sprint 5 SEC-1)."""

from __future__ import annotations

from arcflow import Agent, Workflow
from arcflow._internal.runtime import get_trace


def test_trace_json_has_no_raw_user_input_values() -> None:
    secret = "super-secret-user-input-xyzzy"
    wf = Workflow("redaction")
    wf.step(Agent(name="a", role="researcher", instructions="work"))
    result = wf.run(secret)
    raw = get_trace(result.run_id)
    blob = raw.summary() + str(raw.steps)
    assert secret not in blob
    for step in raw.steps:
        for mem in step.memory_operations:
            assert secret not in mem.key or mem.key != secret
