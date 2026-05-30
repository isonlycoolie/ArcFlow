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

