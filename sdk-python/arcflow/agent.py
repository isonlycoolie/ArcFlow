"""Agent — behavioral unit handed to a Workflow."""

from __future__ import annotations

from uuid import UUID, uuid4

from arcflow.exceptions import WorkflowConfigurationError


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
    ) -> None:
        self.name = _require_non_empty("name", name)
        self.role = _require_non_empty("role", role)
        self.instructions = _require_non_empty("instructions", instructions)
        self.model = model.strip() or "default"
        self.agent_id: UUID = uuid4()

    def __repr__(self) -> str:
        return f"Agent(name={self.name!r}, role={self.role!r})"

    def binding_tuple(self) -> tuple[str, str, str, str]:
        """Serializes agent fields for the native binding layer."""
        return (
            str(self.agent_id),
            self.name,
            self.role,
            self.instructions,
        )
