"""Agent — behavioral unit handed to a Workflow."""

from __future__ import annotations

from uuid import UUID, uuid4

from arcflow.context import ContextPolicy, ToolExecutionConfig
from arcflow.exceptions import WorkflowConfigurationError
from arcflow.memory import MemoryConfig
from arcflow.tool import Tool


def _require_non_empty(field: str, value: str) -> str:
    trimmed = value.strip()
    if not trimmed:
        raise WorkflowConfigurationError(
            f"[ArcFlow] Agent {field} must be a non-empty string. "
            f"Provide a meaningful {field} for this agent."
        )
    return trimmed


class Agent:
    """Defines an agent's identity and instructions (no direct execution)."""

    def __init__(
        self,
        name: str,
        role: str,
        instructions: str,
        model: str = "default",
        tools: tuple[Tool, ...] = (),
        memory: MemoryConfig | None = None,
        context: ContextPolicy | None = None,
        tool_execution: ToolExecutionConfig | None = None,
    ) -> None:
        self.name = _require_non_empty("name", name)
        self.role = _require_non_empty("role", role)
        self.instructions = _require_non_empty("instructions", instructions)
        self.model = model.strip() or "default"
        self.tools = tuple(tools)
        self.memory = memory
        self.context = context
        self.tool_execution = tool_execution
        self._validate_tools()
        self.agent_id: UUID = uuid4()

    def __repr__(self) -> str:
        return f"Agent(name={self.name!r}, role={self.role!r})"

    def _validate_tools(self) -> None:
        seen: set[str] = set()
        for tool in self.tools:
            if tool.name in seen:
                raise WorkflowConfigurationError(
                    f"[ArcFlow] Duplicate tool name '{tool.name}' "
                    f"on agent '{self.name}'."
                )
            seen.add(tool.name)

    def binding_tuple(
        self,
    ) -> tuple[
        str,
        str,
        str,
        str,
        list[tuple[str, str, str, float]],
        str | None,
        str | None,
        str | None,
    ]:
        """Serializes agent fields for the native binding layer."""
        tool_rows = [t.binding_spec() for t in self.tools]
        memory_json = self.memory.binding_json() if self.memory else None
        context_json = self.context.binding_json() if self.context else None
        tool_exec_json = (
            self.tool_execution.binding_json() if self.tool_execution else None
        )
        return (
            str(self.agent_id),
            self.name,
            self.role,
            self.instructions,
            tool_rows,
            memory_json,
            context_json,
            tool_exec_json,
        )
