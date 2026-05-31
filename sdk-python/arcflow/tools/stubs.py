"""Low-level implementations for built-in agent tools."""

from __future__ import annotations

import json
import os
import urllib.parse
import urllib.request


def local_only_blocks_remote() -> bool:
    return os.environ.get("ARCFLOW_EMBEDDING_LOCAL_ONLY", "").strip() == "1"


def web_search(query: str) -> str:
    """Search the web via Tavily or Serper when API keys are configured."""
    if local_only_blocks_remote():
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
