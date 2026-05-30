"""workflow.run_stream() live event emission."""

from __future__ import annotations

import asyncio

import pytest

from arcflow import Agent, Workflow
from arcflow._internal.exec_config import build_exec_config_json
from arcflow._internal import runtime


@pytest.mark.asyncio
async def test_open_workflow_stream_emits_step_start_before_finalize() -> None:
    agent = Agent(name="writer", role="author", instructions="Write.")
    wf = Workflow("stream_wf").step(agent)
    steps, step_rows = wf._agents_and_steps()
    iterator = runtime.open_workflow_stream(
        wf._name,
        "00000000-0000-4000-8000-000000000001",
        steps,
        step_rows,
        "hello",
        None,
        build_exec_config_json(
            retry=None,
            workflow_timeout_seconds=None,
            step_timeout_seconds=None,
            recovery_enabled=False,
            test={"stub_responses": {"step_1": {"output": "fixed"}}},
            stream=True,
        ),
        None,
    )
    event_types: list[str] = []
    loop = asyncio.get_running_loop()
    while True:
        event = await loop.run_in_executor(None, iterator.poll_event)
        if event is None:
            break
        event_types.append(str(event.get("type")))
    result = await loop.run_in_executor(None, iterator.finalize)
    assert "step_start" in event_types
    assert result.output == "fixed"


@pytest.mark.asyncio
async def test_run_stream_public_api_emits_step_start() -> None:
    agent = Agent(name="writer", role="author", instructions="Write.")
    wf = Workflow("stream_api").step(agent)
    gen = wf.run_stream("hello")
    event_types: list[str] = []
    async for event in gen:
        event_types.append(event.type)
    assert "step_start" in event_types
    assert wf._last_run_id
    assert wf._last_result is not None
    assert wf._last_result.output
