"""Workflow — ordered pipeline of Agent steps."""

from __future__ import annotations

from uuid import uuid4

from arcflow.agent import Agent
from arcflow.constants import RETRY_MAX_ALLOWED_ATTEMPTS
from arcflow.exceptions import TraceNotFoundError, WorkflowConfigurationError
from arcflow.provider import ProviderConfig
from arcflow.result import WorkflowResult
from arcflow.retry import BackoffStrategy, ExponentialBackoff
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
        self._step_rows: list[tuple[str, str, int, str | None]] = []
        self._workflow_id: str | None = None
        self._last_run_id: str | None = None
        self._has_run = False
        self._retry: tuple[int, BackoffStrategy] | None = None
        self._workflow_timeout_seconds: float | None = None
        self._step_timeout_seconds: float | None = None
        self._recovery_enabled = False

    def step(self, agent: Agent, *, fallback: Agent | None = None) -> Workflow:
        if not isinstance(agent, Agent):
            raise WorkflowConfigurationError(
                "[ArcFlow] workflow.step() requires an Agent instance. "
                "Pass an arcflow.Agent, not a string or dict."
            )
        if fallback is not None and not isinstance(fallback, Agent):
            raise WorkflowConfigurationError(
                "[ArcFlow] workflow.step(fallback=) requires an Agent instance."
            )
        fallback_step_id: str | None = None
        if fallback is not None:
            for sid, aid, _, _ in self._step_rows:
                if aid == str(fallback.agent_id):
                    fallback_step_id = sid
                    break
            if fallback_step_id is None:
                raise WorkflowConfigurationError(
                    "[ArcFlow] fallback agent must be registered in an earlier workflow.step()."
                )
        step_id = str(uuid4())
        order = len(self._step_rows) + 1
        self._step_rows.append((step_id, str(agent.agent_id), order, fallback_step_id))
        self._steps.append(agent)
        return self

    def retry(
        self,
        max_attempts: int,
        *,
        backoff: BackoffStrategy | None = None,
    ) -> Workflow:
        if self._has_run:
            raise WorkflowConfigurationError(
                "[ArcFlow] workflow.retry() must be called before workflow.run()."
            )
        if max_attempts < 1:
            raise WorkflowConfigurationError(
                f"[ArcFlow] retry max_attempts must be at least 1. Got {max_attempts}."
            )
        if max_attempts > RETRY_MAX_ALLOWED_ATTEMPTS:
            raise WorkflowConfigurationError(
                f"[ArcFlow] retry max_attempts exceeds maximum {RETRY_MAX_ALLOWED_ATTEMPTS}."
            )
        self._retry = (max_attempts, backoff or ExponentialBackoff())
        return self

    def timeout(self, seconds: float) -> Workflow:
        if self._has_run:
            raise WorkflowConfigurationError(
                "[ArcFlow] workflow.timeout() must be called before workflow.run()."
            )
        if seconds <= 0:
            raise WorkflowConfigurationError(
                f"[ArcFlow] Workflow timeout must be positive. Got {seconds}s."
            )
        self._workflow_timeout_seconds = seconds
        return self

    def step_timeout(self, seconds: float) -> Workflow:
        if self._has_run:
            raise WorkflowConfigurationError(
                "[ArcFlow] workflow.step_timeout() must be called before workflow.run()."
            )
        if seconds <= 0:
            raise WorkflowConfigurationError(
                f"[ArcFlow] Step timeout must be positive. Got {seconds}s."
            )
        self._step_timeout_seconds = seconds
        return self

    def enable_recovery(self, storage: str = "postgresql") -> Workflow:
        if storage != "postgresql":
            raise WorkflowConfigurationError(
                f"[ArcFlow] '{storage}' is not a supported recovery backend. "
                "Supported: postgresql."
            )
        self._recovery_enabled = True
        return self

    def resume(self, run_id: str) -> WorkflowResult:
        if not self._recovery_enabled:
            raise WorkflowConfigurationError(
                "[ArcFlow] workflow.resume() requires enable_recovery()."
            )
        if not run_id.strip():
            raise WorkflowConfigurationError(
                "[ArcFlow] resume() requires a non-empty run_id."
            )
        from arcflow._internal.exec_config import build_exec_config_json
        from arcflow._internal import runtime

        if self._workflow_id is None:
            raise WorkflowConfigurationError(
                "[ArcFlow] Cannot resume — no prior run on this workflow instance."
            )
        exec_json = build_exec_config_json(
            retry=self._retry,
            workflow_timeout_seconds=self._workflow_timeout_seconds,
            step_timeout_seconds=self._step_timeout_seconds,
            recovery_enabled=True,
        )
        result = runtime.resume_workflow(
            self._name,
            self._workflow_id,
            self._steps,
            self._step_rows,
            run_id.strip(),
            exec_json,
        )
        self._last_run_id = result.run_id
        return result

    def run(
        self,
        input: str,
        *,
        provider: ProviderConfig | None = None,
    ) -> WorkflowResult:
        trimmed = input.strip()
        if not trimmed:
            raise WorkflowConfigurationError(
                "[ArcFlow] Workflow input must be a non-empty string."
            )
        if not self._steps:
            raise WorkflowConfigurationError(
                "[ArcFlow] Cannot run a workflow with no steps."
            )
        from arcflow._internal.exec_config import build_exec_config_json
        from arcflow._internal import runtime

        exec_json = build_exec_config_json(
            retry=self._retry,
            workflow_timeout_seconds=self._workflow_timeout_seconds,
            step_timeout_seconds=self._step_timeout_seconds,
            recovery_enabled=self._recovery_enabled,
        )
        provider_row = provider.binding_tuple() if provider is not None else None
        if self._workflow_id is None:
            self._workflow_id = str(uuid4())
        result = runtime.run_workflow(
            self._name,
            self._workflow_id,
            self._steps,
            self._step_rows,
            trimmed,
            provider_row,
            exec_json,
        )
        self._last_run_id = result.run_id
        self._has_run = True
        return result

    def trace(self) -> TraceResult:
        if not self._last_run_id:
            raise TraceNotFoundError(
                "[ArcFlow] No workflow run yet. Call workflow.run() before trace()."
            )
        from arcflow._internal import runtime

        return runtime.get_trace(self._last_run_id)
