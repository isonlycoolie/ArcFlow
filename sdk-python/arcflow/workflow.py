"""Workflow — ordered pipeline of Agent steps or graph DAG."""

from __future__ import annotations

import json
from uuid import uuid4

from arcflow.agent import Agent
from arcflow.constants import RETRY_MAX_ALLOWED_ATTEMPTS
from arcflow.exceptions import TraceNotFoundError, WorkflowConfigurationError
from arcflow.hitl import HitlConfig
from arcflow.provider import ProviderConfig
from arcflow.result import WorkflowResult
from arcflow.retry import BackoffStrategy, ExponentialBackoff
from arcflow.trace import TraceResult


class Workflow:
    """Declares steps or graph nodes; execution is delegated to the ArcFlow runtime."""

    def __init__(self, name: str = "default", *, graph: bool = False, runtime: str | None = None) -> None:
        trimmed = name.strip()
        if not trimmed:
            raise WorkflowConfigurationError(
                "[ArcFlow] Workflow name must be a non-empty string. "
                "Provide a descriptive name (e.g. 'research_pipeline')."
            )
        self._name = trimmed
        self._graph_mode = graph
        self._steps: list[Agent] = []
        self._step_rows: list[tuple[str, str, int, str | None, str | None]] = []
        self._graph_nodes: dict[str, tuple[Agent, str]] = {}
        self._graph_edges: list[tuple[str, str | None, str | None]] = []
        self._graph_joins: list[tuple[str, list[str]]] = []
        self._entry_node: str | None = None
        self._max_iterations = 100
        self._workflow_id: str | None = None
        self._last_run_id: str | None = None
        self._has_run = False
        self._retry: tuple[int, BackoffStrategy] | None = None
        self._workflow_timeout_seconds: float | None = None
        self._step_timeout_seconds: float | None = None
        self._recovery_enabled = False
        self._runtime_url: str | None = (
            runtime.strip().rstrip("/") if runtime and runtime.strip() else None
        )

    def step(
        self,
        agent: Agent,
        *,
        fallback: Agent | None = None,
        hitl: HitlConfig | None = None,
    ) -> Workflow:
        if self._graph_mode:
            raise WorkflowConfigurationError(
                "[ArcFlow] Graph workflows use node() — step() is not allowed when graph=True."
            )
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
            for sid, aid, _, _, _ in self._step_rows:
                if aid == str(fallback.agent_id):
                    fallback_step_id = sid
                    break
            if fallback_step_id is None:
                raise WorkflowConfigurationError(
                    "[ArcFlow] fallback agent must be registered in an earlier workflow.step()."
                )
        step_id = str(uuid4())
        order = len(self._step_rows) + 1
        hitl_json = hitl.to_json() if hitl is not None else None
        self._step_rows.append((step_id, str(agent.agent_id), order, fallback_step_id, hitl_json))
        self._steps.append(agent)
        return self

    def node(self, node_id: str, agent: Agent) -> Workflow:
        if not self._graph_mode:
            raise WorkflowConfigurationError(
                "[ArcFlow] node() requires Workflow(graph=True)."
            )
        if self._has_run:
            raise WorkflowConfigurationError(
                "[ArcFlow] workflow.node() must be called before workflow.run()."
            )
        trimmed = node_id.strip()
        if not trimmed:
