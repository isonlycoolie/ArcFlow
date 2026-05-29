"""Unit tests for arcflow_langchain (no langchain-core required for mocks)."""

from __future__ import annotations

import json
from typing import Any

import pytest

from arcflow import Tool, Workflow
from arcflow.exceptions import WorkflowConfigurationError


class _MockLangChainTool:
    name = "mock_search"
    description = "Search mock"
    args_schema = None

    def _run(self, query: str) -> str:
        return f"ok:{query}"


class _MockPrompt:
    template = "Answer: {question}"
    input_variables = ["question"]

    def format(self, **kwargs: object) -> str:
        return self.template.format(**kwargs)


class _MockLangGraph:
    entry_point = "a"

    def __init__(self) -> None:
        self.nodes = {"a": {}, "b": {}, "c": {}}
        self.edges = [
            ("a", "b"),
            ("b", "c"),
            ("c", "__end__"),
        ]


def test_import_without_langchain_core(monkeypatch: pytest.MonkeyPatch) -> None:
    import builtins

    real_import = builtins.__import__

    def _fake_import(name: str, *args: object, **kwargs: object) -> object:
        if name == "langchain_core":
            raise ImportError("no langchain")
        return real_import(name, *args, **kwargs)

    monkeypatch.setattr(builtins, "__import__", _fake_import)
    from arcflow_langchain._deps import require_langchain_core

    with pytest.raises(ImportError, match=r"langchain-core"):
        require_langchain_core()


def test_from_langchain_tool_mock(monkeypatch: pytest.MonkeyPatch) -> None:
    import arcflow_langchain.adapter as adapter_mod

    monkeypatch.setattr(adapter_mod, "require_langchain_core", lambda: None)
    tool = adapter_mod.from_langchain_tool(_MockLangChainTool())
    assert isinstance(tool, Tool)
    assert tool.execute({"query": "hello"}) == "ok:hello"


def test_to_arcflow_step_mock(monkeypatch: pytest.MonkeyPatch) -> None:
    import arcflow_langchain.prompts as prompts_mod

    monkeypatch.setattr(prompts_mod, "require_langchain_core", lambda: None)
    agent = prompts_mod.to_arcflow_step(_MockPrompt(), name="answer")
    assert agent.name == "answer"
    assert "{question}" in agent.instructions


def test_langgraph_three_node_rcs(monkeypatch: pytest.MonkeyPatch) -> None:
    import arcflow_langchain.langgraph_convert as lg_mod

    monkeypatch.setattr(lg_mod, "require_langchain_core", lambda: None)
    wf = lg_mod.langgraph_to_arcflow(_MockLangGraph(), workflow_name="demo")
    assert isinstance(wf, Workflow)
    raw = lg_mod.langgraph_to_rcs_json(_MockLangGraph(), workflow_name="demo")
    body = json.loads(raw)
    assert body["execution_mode"] == "graph"
    assert body["name"] == "demo"
    assert len(body["steps"]) == 3
    graph = body["graph"]
    assert graph["entry_node"] == "a"
    assert len(graph["nodes"]) == 3
    assert any(e.get("from") == "a" and e.get("to") == "b" for e in graph["edges"])


def test_langgraph_empty_nodes_raises(monkeypatch: pytest.MonkeyPatch) -> None:
    import arcflow_langchain.langgraph_convert as lg_mod

    monkeypatch.setattr(lg_mod, "require_langchain_core", lambda: None)
    with pytest.raises(WorkflowConfigurationError, match=r"at least one node"):
        lg_mod.langgraph_to_arcflow(type("G", (), {"nodes": {}})())
