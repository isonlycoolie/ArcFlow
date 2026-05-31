"""Agent context and tool-loop configuration (Phase 2-Pro / RCS v0.6)."""

from __future__ import annotations

import json
from enum import Enum

from arcflow.exceptions import WorkflowConfigurationError


class PriorStepsMode(str, Enum):
    """Which prior step outputs to include in agent context."""

    ALL = "all"
    LAST = "last"
    NONE = "none"


class ContextPolicy:
    """Controls how prior steps and run input are assembled into agent prompts."""

    def __init__(
        self,
        *,
        include_prior_steps: PriorStepsMode | str = PriorStepsMode.ALL,
        include_run_input: bool = True,
        max_prior_step_chars: int = 4096,
    ) -> None:
        if isinstance(include_prior_steps, str):
            try:
                include_prior_steps = PriorStepsMode(include_prior_steps.lower())
            except ValueError as exc:
                raise WorkflowConfigurationError(
                    "[ArcFlow] include_prior_steps must be 'all', 'last', or 'none'."
                ) from exc
        if max_prior_step_chars < 256:
            raise WorkflowConfigurationError(
                "[ArcFlow] max_prior_step_chars must be at least 256."
            )
        self.include_prior_steps = include_prior_steps
        self.include_run_input = include_run_input
        self.max_prior_step_chars = max_prior_step_chars

    def binding_json(self) -> str:
        payload = {
            "include_prior_steps": self.include_prior_steps.value,
            "include_run_input": self.include_run_input,
            "max_prior_step_chars": self.max_prior_step_chars,
        }
        return json.dumps(payload)


class ToolExecutionConfig:
    """Bounds LLM function-calling tool loops on an agent."""

    def __init__(
        self,
        *,
        mode: str = "llm_select",
        max_iterations: int = 5,
    ) -> None:
        if mode not in ("legacy_eager", "llm_select"):
            raise WorkflowConfigurationError(
                "[ArcFlow] tool_execution.mode must be 'legacy_eager' or 'llm_select'."
            )
        if max_iterations < 1 or max_iterations > 20:
            raise WorkflowConfigurationError(
                "[ArcFlow] tool_execution.max_iterations must be between 1 and 20."
            )
        self.mode = mode
        self.max_iterations = max_iterations

    def binding_json(self) -> str:
        return json.dumps({"mode": self.mode, "max_iterations": self.max_iterations})
