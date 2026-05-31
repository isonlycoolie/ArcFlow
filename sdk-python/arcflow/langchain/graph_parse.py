"""LangGraph graph introspection helpers."""

from __future__ import annotations

from typing import Any

from arcflow.exceptions import WorkflowConfigurationError


def graph_view(state_graph: Any) -> Any:
    if hasattr(state_graph, "get_graph") and callable(state_graph.get_graph):
        return state_graph.get_graph()
    if hasattr(state_graph, "graph"):
        return state_graph.graph
    return state_graph


def iter_nodes(graph: Any) -> list[str]:
    nodes = getattr(graph, "nodes", None)
    if nodes is None:
        raise WorkflowConfigurationError(
            "[ArcFlow] LangGraph object has no nodes to convert."
        )
    if isinstance(nodes, dict):
        return [str(k) for k in nodes.keys()]
    return [str(n) for n in nodes]


def iter_edges(graph: Any) -> list[tuple[str, str | None, str | None]]:
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


def entry_node(state_graph: Any, graph: Any, node_ids: list[str]) -> str:
    for attr in ("entry_point", "first_node", "start_node"):
        val = getattr(state_graph, attr, None) or getattr(graph, attr, None)
        if val and str(val) in node_ids:
            return str(val)
    if node_ids:
        return node_ids[0]
    raise WorkflowConfigurationError(
        "[ArcFlow] LangGraph conversion found no nodes."
    )
