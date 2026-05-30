"""Workflow registry HTTP client unit tests."""

from __future__ import annotations

import json

import pytest

from arcflow import Agent, Workflow

pytest.importorskip("pytest_httpserver")
from pytest_httpserver import HTTPServer  # noqa: E402


def _workflow() -> Workflow:
    wf = Workflow("research-agent", runtime="http://127.0.0.1:8080")
    wf.step(Agent(name="writer", role="author", instructions="Write briefly."))
    return wf


def test_publish_calls_put_endpoint(httpserver: HTTPServer, monkeypatch: pytest.MonkeyPatch) -> None:
    wf = _workflow()
    wf._runtime_url = httpserver.url_for("")  # noqa: SLF001
    captured: dict[str, object] = {}

    def handler(request):  # type: ignore[no-untyped-def]
        captured["path"] = request.path
        captured["body"] = json.loads(request.data.decode("utf-8"))
        return json.dumps(
            {
                "name": "research-agent",
                "version": "1.2.0",
                "schema_hash": "sha256:abc",
                "published_at": "2026-05-29T00:00:00Z",
            }
        )

    httpserver.expect_request(
        "/v1/workflows/research-agent/versions/1.2.0", method="PUT"
    ).respond_with_handler(handler)
    monkeypatch.setenv("ARCFLOW_SERVER_API_KEY", "test-key")

    result = wf.publish("1.2.0", published_by="ci")
    assert result["version"] == "1.2.0"
    assert captured["path"] == "/v1/workflows/research-agent/versions/1.2.0"
    body = captured["body"]
    assert isinstance(body, dict)
    assert body.get("published_by") == "ci"
    assert "workflow" in body


def test_resolve_uses_range_query(httpserver: HTTPServer, monkeypatch: pytest.MonkeyPatch) -> None:
    def handler(request):  # type: ignore[no-untyped-def]
        assert request.args.get("range") == "^1.2.0"
        return json.dumps(
            {
                "name": "research-agent",
                "version": "1.3.1",
                "schema_hash": "sha256:def",
                "definition": {"workflow": {}, "agents": []},
            }
        )

    httpserver.expect_request(
        "/v1/workflows/research-agent/resolve",
        method="GET",
    ).respond_with_handler(handler)
    monkeypatch.setenv("ARCFLOW_SERVER_API_KEY", "test-key")

    resolved = Workflow.resolve(
        "research-agent", "^1.2.0", runtime=httpserver.url_for("")
    )
    payload = resolved._build_run_payload("hello", None)  # noqa: SLF001
    assert payload["workflow_ref"] == {
        "name": "research-agent",
        "version": "1.3.1",
    }
