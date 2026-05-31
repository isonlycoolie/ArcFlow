"""Heuristic LangGraph / StateGraph → ArcFlow Workflow conversion."""

from __future__ import annotations

import json
from typing import Any
from uuid import uuid4

from arcflow import Agent, Workflow
from arcflow.exceptions import WorkflowConfigurationError

from arcflow.langchain.deps import require_langchain_core
from arcflow.langchain.graph_parse import entry_node, graph_view, iter_edges, iter_nodes

_UNSUPPORTED = (
    "Dynamic edges, custom checkpointers, and subgraph compilation "
    "are not converted automatically; adjust the ArcFlow graph manually."
)


def langgraph_to_arcflow(
    state_graph: Any,
    *,
    workflow_name: str = "langgraph_import",
    default_role: str = "agent",
    max_iterations: int = 100,
) -> Workflow:
    """Heuristically convert a compiled LangGraph graph into an ArcFlow graph workflow."""
    require_langchain_core()
    graph = graph_view(state_graph)
    node_ids = iter_nodes(graph)
    if not node_ids:
        raise WorkflowConfigurationError(
            "[ArcFlow] LangGraph conversion requires at least one node."
        )

    entry = entry_node(state_graph, graph, node_ids)
    wf = Workflow(workflow_name.strip() or "langgraph_import", graph=True)
    wf.max_iterations(max_iterations)

    for node_id in node_ids:
        agent = Agent(
            name=node_id,
            role=default_role,
            instructions=(
                f"Converted from LangGraph node '{node_id}'. "
                f"Replace with domain-specific instructions. {_UNSUPPORTED}"
            ),
        )
        wf.node(node_id, agent)

    wf.set_entry(entry)
    added_edges = False
    for src, dst, cond in iter_edges(graph):
        if src not in node_ids:
            continue
        if dst is not None and dst not in node_ids:
            continue
        wf.add_edge(src, dst, condition=cond)
        added_edges = True

    if not added_edges:
        ordered = node_ids
        for i, node_id in enumerate(ordered):
            nxt = ordered[i + 1] if i + 1 < len(ordered) else None
            wf.add_edge(node_id, nxt)

    return wf


def langgraph_to_rcs_json(
    state_graph: Any,
    *,
    workflow_name: str = "langgraph_import",
    workflow_id: str | None = None,
) -> str:
    """Return RCS-shaped workflow JSON (execution_mode=graph) for validation."""
    wf = langgraph_to_arcflow(state_graph, workflow_name=workflow_name)
    payload = wf._build_run_payload("migrate", None)
    workflow_body = payload["workflow"]
    if workflow_id:
        workflow_body["id"] = workflow_id
    else:
        workflow_body["id"] = str(uuid4())
    return json.dumps(workflow_body, indent=2)
