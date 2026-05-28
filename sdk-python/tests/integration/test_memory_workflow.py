"""Workflow-level memory and error integration tests (Sprint 4)."""

from __future__ import annotations

import os

import pytest

from arcflow import Agent, MemoryConfig, MemoryScope, MemoryType, Tool, Workflow
from arcflow.exceptions import InfrastructureUnavailableError, ToolExecutionError


def test_session_memory_emits_trace_events() -> None:
    agent = Agent(
        name="mem_agent",
        role="researcher",
        instructions="Use session memory.",
        memory=MemoryConfig(MemoryType.SESSION, MemoryScope.AGENT),
    )
    result = Workflow("mem-wf").step(agent).run("hello-memory")
    kinds = {event.get("event_kind") for event in result.trace_events}
    assert "MemoryWrite" in kinds
    assert "MemoryRead" in kinds


def test_shared_memory_visible_to_second_agent() -> None:
    writer = Agent(
        name="writer",
        role="researcher",
        instructions="Write shared context.",
        memory=MemoryConfig(MemoryType.SHARED, MemoryScope.WORKFLOW),
    )
    reader = Agent(
        name="reader",
        role="researcher",
        instructions="Read shared context.",
        memory=MemoryConfig(MemoryType.SHARED, MemoryScope.WORKFLOW),
    )
    result = Workflow("shared-wf").step(writer).step(reader).run("shared-payload")
    assert result.step_count == 2
    assert "memory_read=" in result.output


def test_unreachable_postgres_raises_infrastructure_unavailable(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setenv(
        "ARCFLOW_POSTGRESQL_URL",
        "postgresql://nobody:nopassword@127.0.0.1:59999/nodb",
    )
    agent = Agent(
        name="persistent_agent",
        role="researcher",
        instructions="Use persistent memory.",
        memory=MemoryConfig(
            MemoryType.PERSISTENT,
            MemoryScope.GLOBAL,
            namespace="failure_test",
        ),
    )
    with pytest.raises(InfrastructureUnavailableError) as exc_info:
        Workflow("infra-wf").step(agent).run("test input")
    assert exc_info.value.backend == "postgresql"


@pytest.mark.skipif(
    os.environ.get("ARCFLOW_RUN_SLOW_TOOL_TESTS") != "1",
    reason="Set ARCFLOW_RUN_SLOW_TOOL_TESTS=1 to run timeout test",
)
def test_tool_timeout_raises_tool_execution_error() -> None:
    import time

    def slow_tool(_: dict) -> str:
        time.sleep(2.0)
        return "late"

    tool = Tool(
        name="slow",
        description="Slow tool.",
        input_schema={"type": "object", "properties": {}},
        execute=slow_tool,
        timeout_seconds=0.1,
    )
    agent = Agent(
        name="tester",
        role="researcher",
        instructions="Call slow tool.",
        tools=(tool,),
    )
    with pytest.raises(ToolExecutionError) as exc_info:
        Workflow("timeout-wf").step(agent).run("test")
    assert exc_info.value.tool_name == "slow"


def test_tool_failure_raises_tool_execution_error() -> None:
    def boom(_: dict) -> str:
        raise ValueError("tool broke")

    tool = Tool(
        name="fail",
        description="Fails on purpose.",
        input_schema={
            "type": "object",
            "properties": {"message": {"type": "string"}},
        },
        execute=boom,
    )
    agent = Agent(
        name="tester",
        role="researcher",
        instructions="Invoke failing tool.",
        tools=(tool,),
    )
    with pytest.raises(ToolExecutionError) as exc_info:
        Workflow("fail-wf").step(agent).run("test")
    assert exc_info.value.tool_name == "fail"
