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
            raise WorkflowConfigurationError(
                "[ArcFlow] Graph node id must be a non-empty string."
            )
        if not isinstance(agent, Agent):
            raise WorkflowConfigurationError(
                "[ArcFlow] workflow.node() requires an Agent instance."
            )
        if trimmed in self._graph_nodes:
            raise WorkflowConfigurationError(
                f"[ArcFlow] Duplicate graph node id '{trimmed}'."
            )
        step_id = str(uuid4())
        self._graph_nodes[trimmed] = (agent, step_id)
        if self._entry_node is None:
            self._entry_node = trimmed
        return self

    def add_edge(
        self,
        from_id: str,
        to_id: str | None = None,
        *,
        condition: str | None = None,
    ) -> Workflow:
        if not self._graph_mode:
            raise WorkflowConfigurationError(
                "[ArcFlow] add_edge() requires Workflow(graph=True)."
            )
        if self._has_run:
            raise WorkflowConfigurationError(
                "[ArcFlow] workflow.add_edge() must be called before workflow.run()."
            )
        self._graph_edges.append((from_id.strip(), to_id, condition))
        return self

    def join_node(self, join_id: str, wait_for: list[str]) -> Workflow:
        if not self._graph_mode:
            raise WorkflowConfigurationError(
                "[ArcFlow] join_node() requires Workflow(graph=True)."
            )
        trimmed = join_id.strip()
        if not trimmed:
            raise WorkflowConfigurationError(
                "[ArcFlow] join_node id must be a non-empty string."
            )
        if trimmed not in self._graph_nodes:
            raise WorkflowConfigurationError(
                f"[ArcFlow] Join node '{trimmed}' is not registered."
            )
        if not wait_for:
            raise WorkflowConfigurationError(
                "[ArcFlow] join_node wait_for must list at least one branch node."
            )
        self._graph_joins.append((trimmed, [b.strip() for b in wait_for]))
        return self

    def set_entry(self, node_id: str) -> Workflow:
        if not self._graph_mode:
            raise WorkflowConfigurationError(
                "[ArcFlow] set_entry() requires Workflow(graph=True)."
            )
        trimmed = node_id.strip()
        if trimmed not in self._graph_nodes:
            raise WorkflowConfigurationError(
                f"[ArcFlow] Entry node '{trimmed}' is not registered."
            )
        self._entry_node = trimmed
        return self

    def max_iterations(self, count: int) -> Workflow:
        if count < 1:
            raise WorkflowConfigurationError(
                "[ArcFlow] max_iterations must be at least 1."
            )
        self._max_iterations = count
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

    def _graph_payload(self) -> str:
        if not self._entry_node:
            raise WorkflowConfigurationError(
                "[ArcFlow] Graph workflow has no entry node."
            )
        nodes = [
            {"id": node_id, "step_id": step_id}
            for node_id, (_, step_id) in self._graph_nodes.items()
        ]
        edges = [
            {"from": f, "to": t, "condition": c}
            for f, t, c in self._graph_edges
        ]
        payload: dict[str, object] = {
            "entry_node": self._entry_node,
            "max_iterations": self._max_iterations,
            "nodes": nodes,
            "edges": edges,
        }
        if self._graph_joins:
            payload["join_nodes"] = [
                {"id": join_id, "wait_for": branches}
                for join_id, branches in self._graph_joins
            ]
        return json.dumps(payload)

    def _agents_and_steps(self) -> tuple[list[Agent], list[tuple[str, str, int, str | None]]]:
        if self._graph_mode:
            agents: list[Agent] = []
            rows: list[tuple[str, str, int, str | None, str | None]] = []
            for order, (node_id, (agent, step_id)) in enumerate(
                self._graph_nodes.items(), start=1
            ):
                agents.append(agent)
                rows.append((step_id, str(agent.agent_id), order, None, None))
            return agents, rows
        return self._steps, self._step_rows

    def _build_run_payload(
        self, run_input: str, exec_config_json: str | None
    ) -> dict[str, object]:
        agents, steps = self._agents_and_steps()
        workflow_id = self._workflow_id or str(uuid4())
        self._workflow_id = workflow_id
        step_defs = []
        for sid, aid, order, _, hitl_json in steps:
            row: dict[str, object] = {"id": sid, "agent_id": aid, "order": order}
            if hitl_json:
                row["hitl"] = json.loads(hitl_json)
            step_defs.append(row)
        agent_defs = [
            {
                "id": str(agent.agent_id),
                "name": agent.name,
                "role": agent.role,
                "instructions": agent.instructions,
            }
            for agent in agents
        ]
        workflow_body: dict[str, object] = {
            "id": workflow_id,
            "name": self._name,
            "steps": step_defs,
            "execution_mode": "graph" if self._graph_mode else "linear",
        }
        if self._graph_mode:
            workflow_body["graph"] = json.loads(self._graph_payload())
        payload: dict[str, object] = {
            "workflow": workflow_body,
            "agents": agent_defs,
            "input": run_input,
        }
        if exec_config_json:
            payload["exec_config"] = json.loads(exec_config_json)
        return payload

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
        steps, step_rows = self._agents_and_steps()
        result = runtime.resume_workflow(
            self._name,
            self._workflow_id,
            steps,
            step_rows,
            run_id.strip(),
            exec_json,
        )
        self._last_run_id = result.run_id
        return result

    def resume_with_approval(
        self,
        run_id: str,
        approval_key: str,
        *,
        approved: bool = True,
        data: dict[str, object] | None = None,
    ) -> WorkflowResult:
        if not self._recovery_enabled:
            raise WorkflowConfigurationError(
                "[ArcFlow] resume_with_approval() requires enable_recovery()."
            )
        if self._runtime_url:
            from arcflow._internal import remote

            return remote.approve_run(
                self,
                run_id.strip(),
                approval_key.strip(),
                approved=approved,
                data=data or {},
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
        steps, step_rows = self._agents_and_steps()
        result = runtime.resume_with_approval(
            self._name,
            self._workflow_id,
            steps,
            step_rows,
            run_id.strip(),
            approval_key.strip(),
            approved,
