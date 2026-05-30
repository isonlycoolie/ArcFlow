"""Heuristic LangGraph / StateGraph → ArcFlow Workflow conversion."""

from __future__ import annotations

import json
from typing import Any
from uuid import uuid4

from arcflow import Agent, Workflow
from arcflow.exceptions import WorkflowConfigurationError

from arcflow_langchain._deps import require_langchain_core

_UNSUPPORTED = (
    "Dynamic edges, custom checkpointers, and subgraph compilation "
    "are not converted automatically — adjust the ArcFlow graph manually."
)


def _graph_view(state_graph: Any) -> Any:
    if hasattr(state_graph, "get_graph") and callable(state_graph.get_graph):
        return state_graph.get_graph()
    if hasattr(state_graph, "graph"):
        return state_graph.graph
    return state_graph


def _iter_nodes(graph: Any) -> list[str]:
    nodes = getattr(graph, "nodes", None)
    if nodes is None:
        raise WorkflowConfigurationError(
            "[ArcFlow] LangGraph object has no nodes to convert."
        )
    if isinstance(nodes, dict):
        return [str(k) for k in nodes.keys()]
    return [str(n) for n in nodes]


def _iter_edges(graph: Any) -> list[tuple[str, str | None, str | None]]:
    raw_edges = getattr(graph, "edges", None)
    if raw_edges is None:
        return []

    parsed: list[tuple[str, str | None, str | None]] = []
    end_markers = frozenset({"__end__", "END", "end"})

    for edge in raw_edges:
        if isinstance(edge, tuple) and len(edge) >= 2:
            src, dst = str(edge[0]), edge[1]
            cond = edge[2] if len(edge) > 2 else None
            to_id = None if dst is None or str(dst) in end_markers else str(dst)
            parsed.append((src, to_id, str(cond) if cond else None))
            continue
        src = getattr(edge, "source", None) or getattr(edge, "from", None)
        dst = getattr(edge, "target", None) or getattr(edge, "to", None)
        cond = getattr(edge, "condition", None) or getattr(edge, "data", None)
        if src is None:
            continue
        to_id = None if dst is None or str(dst) in end_markers else str(dst)
        cond_str = str(cond) if cond is not None else None
        parsed.append((str(src), to_id, cond_str))
    return parsed


def _entry_node(state_graph: Any, graph: Any, node_ids: list[str]) -> str:
    for attr in ("entry_point", "first_node", "start_node"):
        val = getattr(state_graph, attr, None) or getattr(graph, attr, None)
        if val and str(val) in node_ids:
            return str(val)
    if node_ids:
        return node_ids[0]
    raise WorkflowConfigurationError(
        "[ArcFlow] LangGraph conversion found no nodes."
    )


def langgraph_to_arcflow(
    state_graph: Any,
    *,
    workflow_name: str = "langgraph_import",
    default_role: str = "agent",
    max_iterations: int = 100,
) -> Workflow:
    """Heuristically convert a compiled LangGraph graph into an ArcFlow graph workflow.

    See module docstring in docs for unsupported patterns. Manual tuning is expected.
    """
    require_langchain_core()
    graph = _graph_view(state_graph)
    node_ids = _iter_nodes(graph)
    if not node_ids:
        raise WorkflowConfigurationError(
            "[ArcFlow] LangGraph conversion requires at least one node."
        )

    entry = _entry_node(state_graph, graph, node_ids)
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
    for src, dst, cond in _iter_edges(graph):
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
