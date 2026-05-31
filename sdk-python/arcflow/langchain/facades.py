"""PascalCase facades for LangChain interop."""

from __future__ import annotations

from typing import Any

from arcflow.agent import Agent
from arcflow.tool import Tool
from arcflow.workflow import Workflow

from arcflow.langchain.graph_convert import langgraph_to_arcflow, langgraph_to_rcs_json
from arcflow.langchain.prompt_adapter import to_arcflow_step
from arcflow.langchain.tool_adapter import from_langchain_tool


class FromLangChain:
    """Import LangChain prompts and tools into ArcFlow types."""

    @staticmethod
    def prompt(
        prompt_template: Any,
        *,
        name: str = "step",
        role: str = "assistant",
        model: str = "default",
    ) -> Agent:
        return to_arcflow_step(
            prompt_template,
            name=name,
            role=role,
            model=model,
        )

    @staticmethod
    def tool(lc_tool: Any, *, timeout_seconds: float = 30.0) -> Tool:
        return from_langchain_tool(lc_tool, timeout_seconds=timeout_seconds)


class LangChainToArcflow:
    """Convert LangGraph compiled graphs into ArcFlow workflows."""

    @staticmethod
    def convert(
        state_graph: Any,
        *,
        workflow_name: str = "langgraph_import",
        default_role: str = "agent",
        max_iterations: int = 100,
    ) -> Workflow:
        return langgraph_to_arcflow(
            state_graph,
            workflow_name=workflow_name,
            default_role=default_role,
            max_iterations=max_iterations,
        )

    @staticmethod
    def to_rcs_json(
        state_graph: Any,
        *,
        workflow_name: str = "langgraph_import",
        workflow_id: str | None = None,
    ) -> str:
        return langgraph_to_rcs_json(
            state_graph,
            workflow_name=workflow_name,
            workflow_id=workflow_id,
        )
