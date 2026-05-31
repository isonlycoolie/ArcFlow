"""Deprecated: use ``arcflow.langchain`` (FromLangChain, LangChainToArcflow)."""

from __future__ import annotations

import warnings

warnings.warn(
    "arcflow_langchain is deprecated; use "
    "from arcflow.langchain import FromLangChain, LangChainToArcflow",
    DeprecationWarning,
    stacklevel=2,
)

from arcflow.langchain import FromLangChain, LangChainToArcflow
from arcflow.langchain.graph_convert import langgraph_to_arcflow, langgraph_to_rcs_json
from arcflow.langchain.prompt_adapter import to_arcflow_step
from arcflow.langchain.tool_adapter import from_langchain_tool

from_langchain_tool = FromLangChain.tool
to_arcflow_step = FromLangChain.prompt
langgraph_to_arcflow = LangChainToArcflow.convert
langgraph_to_rcs_json = LangChainToArcflow.to_rcs_json

__all__ = [
    "FromLangChain",
    "LangChainToArcflow",
    "from_langchain_tool",
    "to_arcflow_step",
    "langgraph_to_arcflow",
    "langgraph_to_rcs_json",
]
