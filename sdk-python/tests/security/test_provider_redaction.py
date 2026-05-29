"""Ensure provider errors and traces never expose API keys."""

from __future__ import annotations

import pytest

from arcflow import Agent, OpenAI, Workflow
from arcflow._arcflow_binding import get_execution_trace_json
from arcflow.exceptions import ProviderExecutionError

pytest.importorskip("pytest_httpserver")
from pytest_httpserver import HTTPServer  # noqa: E402


def test_provider_error_message_redacts_api_key(
    httpserver: HTTPServer, monkeypatch: pytest.MonkeyPatch
) -> None:
    secret = "sk-test-secret-key-not-real-abcdef1234567890"
    httpserver.expect_request("/v1/chat/completions", method="POST").respond_with_data(
        "unauthorized", status=401
    )
    monkeypatch.setenv("OPENAI_API_KEY", secret)
    monkeypatch.setenv(
        "ARCFLOW_OPENAI_API_ENDPOINT", httpserver.url_for("/v1/chat/completions")
    )
    wf = Workflow("auth-fail")
    wf.step(Agent(name="a", role="writer", instructions="Write."))
    with pytest.raises(ProviderExecutionError) as exc:
        wf.run("hello", provider=OpenAI(model="gpt-4o"))
    message = str(exc.value)
    assert secret not in message
    assert "OPENAI_API_KEY" in message or "Authentication" in message or "401" in message


def test_trace_blob_never_contains_openai_key(
    httpserver: HTTPServer, monkeypatch: pytest.MonkeyPatch
) -> None:
    secret = "sk-test-secret-key-not-real-abcdef1234567890"
    httpserver.expect_request("/v1/chat/completions", method="POST").respond_with_json(
        {
            "model": "gpt-4o",
            "choices": [{"message": {"content": "ok"}, "finish_reason": "stop"}],
            "usage": {"prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2},
        }
    )
    monkeypatch.setenv("OPENAI_API_KEY", secret)
    monkeypatch.setenv(
        "ARCFLOW_OPENAI_API_ENDPOINT", httpserver.url_for("/v1/chat/completions")
    )
    result = (
        Workflow("trace-redaction")
        .step(Agent(name="a", role="writer", instructions="Write."))
        .run("hello", provider=OpenAI(model="gpt-4o"))
    )
    assert secret not in get_execution_trace_json(result.run_id)
