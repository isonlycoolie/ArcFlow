"""Workflow — ordered pipeline of Agent steps."""

from __future__ import annotations

from arcflow.agent import Agent
from arcflow.exceptions import TraceNotFoundError, WorkflowConfigurationError
from arcflow.result import WorkflowResult
from arcflow.trace import TraceResult


class Workflow:
    """Declares steps; execution is delegated to the ArcFlow runtime."""

    def __init__(self, name: str = "default") -> None:
        trimmed = name.strip()
        if not trimmed:
            raise WorkflowConfigurationError(
                "[ArcFlow] Workflow name must be a non-empty string. "
                "Provide a descriptive name (e.g. 'research_pipeline')."
            )
        self._name = trimmed
        self._steps: list[Agent] = []
        self._last_run_id: str | None = None

    def step(self, agent: Agent) -> Workflow:
        if not isinstance(agent, Agent):
            raise WorkflowConfigurationError(
                "[ArcFlow] workflow.step() requires an Agent instance. "
                "Pass an arcflow.Agent, not a string or dict."
            )
        self._steps.append(agent)
        return self

    def run(self, input: str) -> WorkflowResult:
        trimmed = input.strip()
        if not trimmed:
            raise WorkflowConfigurationError(
                "[ArcFlow] Workflow input must be a non-empty string. "
                "Provide the initial text passed to the first step."
            )
        if not self._steps:
            raise WorkflowConfigurationError(
                "[ArcFlow] Cannot run a workflow with no steps. "
                "Add at least one step with workflow.step(agent) before calling run()."
            )
        from arcflow._internal import runtime

        result = runtime.run_workflow(self._name, self._steps, trimmed)
        self._last_run_id = result.run_id
        return result

    def trace(self) -> TraceResult:
        """Returns the trace of the most recent ``run()`` on this workflow."""
        if not self._last_run_id:
            raise TraceNotFoundError(
                "[ArcFlow] No workflow run yet. Call workflow.run() before trace()."
            )
        from arcflow._internal import runtime

        return runtime.get_trace(self._last_run_id)
