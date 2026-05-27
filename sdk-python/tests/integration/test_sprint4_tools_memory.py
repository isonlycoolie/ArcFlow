"""Sprint 4 tools and memory integration (Python SDK)."""

from __future__ import annotations

import pytest

from arcflow import Agent, MemoryConfig, MemoryScope, MemoryType, Tool, Workflow
from arcflow.exceptions import MemoryConfigurationError, ToolConfigurationError


def test_tool_configuration_rejects_invalid_schema() -> None:
    with pytest.raises(ToolConfigurationError, match=r"\[ArcFlow\]"):
        Tool(
            name="t",
            description="d",
            input_schema="not-a-dict",  # type: ignore[arg-type]
            execute=lambda _: "x",
        )


def test_workflow_with_tool_echoes_message() -> None:
    def echo(payload: dict) -> str:
        return str(payload.get("message", ""))

    tool = Tool(
        name="echo",
        description="echo",
        input_schema={
            "type": "object",
            "properties": {"message": {"type": "string"}},
        },
        execute=echo,
    )
    agent = Agent(
        name="worker",
        role="researcher",
        instructions="run tool",
        tools=(tool,),
    )
    wf = Workflow("tool-wf").step(agent)
    result = wf.run("hello-tools")
    assert result.step_count == 1
    assert "hello-tools" in result.output or "researcher" in result.output


def test_memory_config_requires_namespace_for_persistent() -> None:
    with pytest.raises(MemoryConfigurationError, match=r"\[ArcFlow\]"):
        MemoryConfig(MemoryType.PERSISTENT, MemoryScope.GLOBAL)
