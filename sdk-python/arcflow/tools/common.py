"""Built-in tools for ArcFlow agents (web search, HTTP fetch, document read)."""

from __future__ import annotations

from arcflow import Tool

from arcflow.tools.stubs import http_fetch, read_document, web_search


def _web_search_tool(args: dict[str, object]) -> str:
    return web_search(str(args.get("query", "")))


def _http_fetch_tool(args: dict[str, object]) -> str:
    return http_fetch(str(args.get("url", "")))


def _read_document_tool(args: dict[str, object]) -> str:
    return read_document(str(args.get("path_or_url", "")))


class CommonTools:
    """Optional web and document tools for Agent(..., tools=)."""

    @staticmethod
    def bundle() -> tuple[Tool, Tool, Tool]:
        """Return web_search, http_fetch, and read_document tools."""
        return (
            Tool(
                "web_search",
                "Search the web for current information.",
                {
                    "type": "object",
                    "properties": {"query": {"type": "string"}},
                    "required": ["query"],
                },
                _web_search_tool,
            ),
            Tool(
                "http_fetch",
                "Fetch a URL on ARCFLOW_HTTP_ALLOWLIST.",
                {
                    "type": "object",
                    "properties": {"url": {"type": "string"}},
                    "required": ["url"],
                },
                _http_fetch_tool,
            ),
            Tool(
                "read_document",
                "Read a local file or allowlisted URL.",
                {
                    "type": "object",
                    "properties": {"path_or_url": {"type": "string"}},
                    "required": ["path_or_url"],
                },
                _read_document_tool,
            ),
        )
