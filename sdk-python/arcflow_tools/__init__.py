"""Optional common tools for ArcFlow agents (Phase 2-Pro)."""

from __future__ import annotations

import json
import os
import urllib.parse
import urllib.request

from arcflow import Tool


def _local_only_blocks_remote() -> bool:
    return os.environ.get("ARCFLOW_EMBEDDING_LOCAL_ONLY", "").strip() == "1"


def web_search(query: str) -> str:
    """Search the web via Tavily or Serper when API keys are configured."""
    if _local_only_blocks_remote():
        return json.dumps({"stub": True, "query": query, "results": []})
    if key := os.environ.get("TAVILY_API_KEY", "").strip():
        body = json.dumps({"api_key": key, "query": query, "max_results": 5}).encode()
        req = urllib.request.Request(
            "https://api.tavily.com/search",
            data=body,
            headers={"Content-Type": "application/json"},
            method="POST",
        )
        with urllib.request.urlopen(req, timeout=30) as resp:
            return resp.read().decode()
    if key := os.environ.get("SERPER_API_KEY", "").strip():
        body = json.dumps({"q": query}).encode()
        req = urllib.request.Request(
            "https://google.serper.dev/search",
            data=body,
            headers={"X-API-KEY": key, "Content-Type": "application/json"},
            method="POST",
        )
        with urllib.request.urlopen(req, timeout=30) as resp:
            return resp.read().decode()
    return json.dumps({"error": "Set TAVILY_API_KEY or SERPER_API_KEY"})


def http_fetch(url: str) -> str:
    """Fetch a URL when the host is on ARCFLOW_HTTP_ALLOWLIST."""
    allow = {
        h.strip().lower()
        for h in os.environ.get("ARCFLOW_HTTP_ALLOWLIST", "").split(",")
        if h.strip()
    }
    parsed = urllib.parse.urlparse(url)
    host = (parsed.hostname or "").lower()
    if not allow or host not in allow:
        return json.dumps({"error": "host not on ARCFLOW_HTTP_ALLOWLIST", "host": host})
    with urllib.request.urlopen(url, timeout=30) as resp:
        return resp.read().decode(errors="replace")[:65536]


def read_document(path_or_url: str) -> str:
    """Read a local file or allowlisted URL."""
    if path_or_url.startswith(("http://", "https://")):
        return http_fetch(path_or_url)
    with open(path_or_url, encoding="utf-8", errors="replace") as f:
        return f.read()[:65536]


def _web_search_tool(args: dict[str, object]) -> str:
    return web_search(str(args.get("query", "")))


def _http_fetch_tool(args: dict[str, object]) -> str:
    return http_fetch(str(args.get("url", "")))


def _read_document_tool(args: dict[str, object]) -> str:
    return read_document(str(args.get("path_or_url", "")))


def common_tools() -> tuple[Tool, Tool, Tool]:
    """Return web_search, http_fetch, and read_document tools for Agent(..., tools=)."""
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


def register_common_tools(_workflow: object) -> tuple[Tool, Tool, Tool]:
    """Deprecated alias — attach tools via Agent(..., tools=common_tools())."""
    return common_tools()
