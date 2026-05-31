"""LangChain BaseTool → ArcFlow Tool adapter."""

from __future__ import annotations

import asyncio
import inspect
from typing import Any

from arcflow.tool import Tool

from arcflow.langchain.deps import require_langchain_core


def _default_input_schema() -> dict[str, Any]:
    return {
        "type": "object",
        "properties": {},
        "additionalProperties": True,
    }


def _schema_from_args(args_schema: Any) -> dict[str, Any]:
    if args_schema is None:
        return _default_input_schema()
    if hasattr(args_schema, "model_json_schema"):
        return args_schema.model_json_schema()
    if hasattr(args_schema, "schema"):
        return args_schema.schema()
    if isinstance(args_schema, dict):
        return args_schema
    return _default_input_schema()


def _normalize_run_result(result: Any) -> str:
    if result is None:
        return ""
    if isinstance(result, str):
        return result
    return str(result)


def from_langchain_tool(lc_tool: Any, *, timeout_seconds: float = 30.0) -> Tool:
    """Wrap a LangChain ``BaseTool`` as an ArcFlow ``Tool``."""
    require_langchain_core()

    name = getattr(lc_tool, "name", None) or getattr(lc_tool, "__name__", None)
    if not name or not str(name).strip():
        raise ValueError("[ArcFlow] LangChain tool must have a non-empty name.")

    description = (
        getattr(lc_tool, "description", None)
        or getattr(lc_tool, "desc", None)
        or str(name)
    )
    input_schema = _schema_from_args(getattr(lc_tool, "args_schema", None))

    def execute(inputs: dict[str, Any]) -> str:
        if hasattr(lc_tool, "_run"):
            sig = inspect.signature(lc_tool._run)
            params = list(sig.parameters.values())
            if len(params) == 1 and params[0].name not in inputs:
                key = params[0].name
                if key in ("tool_input", "query", "input"):
                    return _normalize_run_result(lc_tool._run(inputs.get(key, inputs)))
            return _normalize_run_result(lc_tool._run(**inputs))

        if hasattr(lc_tool, "_arun"):

            async def _call() -> Any:
                sig = inspect.signature(lc_tool._arun)
                params = list(sig.parameters.values())
                if len(params) == 1 and params[0].name not in inputs:
                    key = params[0].name
                    if key in ("tool_input", "query", "input"):
                        return await lc_tool._arun(inputs.get(key, inputs))
                return await lc_tool._arun(**inputs)

            return _normalize_run_result(asyncio.run(_call()))

        if hasattr(lc_tool, "invoke"):
            return _normalize_run_result(lc_tool.invoke(inputs))

        raise TypeError(
            "[ArcFlow] LangChain tool must implement _run, _arun, or invoke."
        )

    return Tool(
        name=str(name),
        description=str(description),
        input_schema=input_schema,
        execute=execute,
        timeout_seconds=timeout_seconds,
    )
