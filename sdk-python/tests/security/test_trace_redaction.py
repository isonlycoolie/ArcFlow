"""Ensure traces never expose sensitive payload values (Sprint 5 SEC-1)."""

from __future__ import annotations

from arcflow import Agent, Tool, Workflow
from arcflow._arcflow_binding import get_execution_trace_json


def _trace_blob(run_id: str) -> str:
    return get_execution_trace_json(run_id)


def test_trace_does_not_contain_agent_instructions() -> None:
    secret = "super-secret-agent-instructions-xyzzy"
    wf = Workflow("redaction-instructions")
    wf.step(Agent(name="a", role="researcher", instructions=secret))
    result = wf.run("hello")
    assert secret not in _trace_blob(result.run_id)


def test_trace_does_not_contain_workflow_input() -> None:
    secret = "super-secret-user-input-xyzzy"
    wf = Workflow("redaction-input")
    wf.step(Agent(name="a", role="researcher", instructions="work"))
    result = wf.run(secret)
    assert secret not in _trace_blob(result.run_id)


def test_tool_output_not_in_trace() -> None:
    secret = "super-secret-tool-output-xyzzy"

    def echo(_payload: dict) -> str:
        return secret

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
    result = Workflow("redaction-tool").step(agent).run("trigger")
    assert secret not in _trace_blob(result.run_id)
