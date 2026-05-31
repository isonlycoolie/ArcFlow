"""Unit tests for arcflow.langchain (no langchain-core required for mocks)."""

from __future__ import annotations

import json
import warnings

import pytest

from arcflow import Tool, Workflow
from arcflow.exceptions import WorkflowConfigurationError
from arcflow.langchain import FromLangChain, LangChainToArcflow


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
    from arcflow.langchain.deps import require_langchain_core

    with pytest.raises(ImportError, match=r"langchain-core"):
        require_langchain_core()


def test_from_langchain_tool_mock(monkeypatch: pytest.MonkeyPatch) -> None:
    import arcflow.langchain.tool_adapter as adapter_mod

    monkeypatch.setattr(adapter_mod, "require_langchain_core", lambda: None)
    tool = FromLangChain.tool(_MockLangChainTool())
    assert isinstance(tool, Tool)
    assert tool.execute({"query": "hello"}) == "ok:hello"


def test_from_langchain_tool_in_workflow_test(monkeypatch: pytest.MonkeyPatch) -> None:
    import arcflow.langchain.tool_adapter as adapter_mod

    monkeypatch.setattr(adapter_mod, "require_langchain_core", lambda: None)
    from arcflow import Agent

    tool = FromLangChain.tool(_MockLangChainTool())
    agent = Agent(
        name="researcher",
        role="analyst",
        instructions="Use search.",
        tools=(tool,),
    )
    wf = Workflow("lc_tool_wf").step(agent)
    results = wf.test(
        [
            {
                "input": "hello",
                "expected_output": "done",
                "stub_responses": {"step_1": {"output": "done"}},
            }
        ]
    )
    assert results[0]["passed"] is True


def test_langgraph_rcs_validates_against_schema(monkeypatch: pytest.MonkeyPatch) -> None:
    from pathlib import Path

    import jsonschema

    import arcflow.langchain.graph_convert as lg_mod

    monkeypatch.setattr(lg_mod, "require_langchain_core", lambda: None)
    raw = LangChainToArcflow.to_rcs_json(_MockLangGraph(), workflow_name="demo")
    body = json.loads(raw)
    schema_path = (
        Path(__file__).resolve().parents[3]
        / "contracts"
        / "normative"
        / "rcs"
        / "v1.schema.json"
    )
    schema = json.loads(schema_path.read_text(encoding="utf-8"))
    jsonschema.validate(instance=body, schema=schema)


def test_canonical_import_no_deprecation_warning() -> None:
    with warnings.catch_warnings():
        warnings.simplefilter("error", DeprecationWarning)
        from arcflow.langchain import FromLangChain as _fl  # noqa: F401

        assert _fl.prompt is not None


def test_deprecated_arcflow_langchain_import_warns() -> None:
    with warnings.catch_warnings(record=True) as caught:
        warnings.simplefilter("always")
        import importlib

        import arcflow_langchain

        importlib.reload(arcflow_langchain)
    assert any(
        issubclass(w.category, DeprecationWarning) for w in caught
    )


def test_to_arcflow_step_mock(monkeypatch: pytest.MonkeyPatch) -> None:
    import arcflow.langchain.prompt_adapter as prompts_mod

    monkeypatch.setattr(prompts_mod, "require_langchain_core", lambda: None)
    agent = FromLangChain.prompt(_MockPrompt(), name="answer")
    assert agent.name == "answer"
    assert "{question}" in agent.instructions


def test_langgraph_three_node_rcs(monkeypatch: pytest.MonkeyPatch) -> None:
    import arcflow.langchain.graph_convert as lg_mod

    monkeypatch.setattr(lg_mod, "require_langchain_core", lambda: None)
    wf = LangChainToArcflow.convert(_MockLangGraph(), workflow_name="demo")
    assert isinstance(wf, Workflow)
    raw = LangChainToArcflow.to_rcs_json(_MockLangGraph(), workflow_name="demo")
    body = json.loads(raw)
    assert body["execution_mode"] == "graph"
    assert body["name"] == "demo"
    assert len(body["steps"]) == 3
    graph = body["graph"]
    assert graph["entry_node"] == "a"
    assert len(graph["nodes"]) == 3
    assert any(e.get("from") == "a" and e.get("to") == "b" for e in graph["edges"])


def test_langgraph_empty_nodes_raises(monkeypatch: pytest.MonkeyPatch) -> None:
    import arcflow.langchain.graph_convert as lg_mod

    monkeypatch.setattr(lg_mod, "require_langchain_core", lambda: None)
    with pytest.raises(WorkflowConfigurationError, match=r"at least one node"):
        LangChainToArcflow.convert(type("G", (), {"nodes": {}})())


def test_common_tools_bundle() -> None:
    from arcflow.tools import CommonTools

    tools = CommonTools.bundle()
    assert len(tools) == 3
    assert {t.name for t in tools} == {"web_search", "http_fetch", "read_document"}


def test_deprecated_arcflow_tools_import_warns() -> None:
    with warnings.catch_warnings(record=True) as caught:
        warnings.simplefilter("always")
        import importlib

        import arcflow_tools

        importlib.reload(arcflow_tools)
    assert any(
        issubclass(w.category, DeprecationWarning) for w in caught
    )
