"""Provider swap integration tests with mock HTTP servers."""

from __future__ import annotations

import json

import pytest

from arcflow import Agent, Anthropic, Gemini, OpenAI, Workflow
from arcflow._arcflow_binding import get_execution_trace_json

pytest.importorskip("pytest_httpserver")
from pytest_httpserver import HTTPServer  # noqa: E402


def _agent() -> Agent:
    return Agent(name="writer", role="author", instructions="Write a short reply.")


def test_openai_provider_swap_matches_stub_step_count(
    httpserver: HTTPServer, monkeypatch: pytest.MonkeyPatch
) -> None:
    httpserver.expect_request("/v1/chat/completions", method="POST").respond_with_json(
        {
            "model": "gpt-4o",
            "choices": [{"message": {"content": "ok"}, "finish_reason": "stop"}],
            "usage": {"prompt_tokens": 3, "completion_tokens": 2, "total_tokens": 5},
        }
    )
    monkeypatch.setenv("OPENAI_API_KEY", "test-key")
    monkeypatch.setenv(
        "ARCFLOW_OPENAI_API_ENDPOINT", httpserver.url_for("/v1/chat/completions")
    )
    stub = Workflow("stub").step(_agent()).run("hello").step_count
    provider = (
        Workflow("openai")
        .step(_agent())
        .run("hello", provider=OpenAI(model="gpt-4o"))
        .step_count
    )
    assert provider == stub == 1


def test_anthropic_provider_swap_matches_stub_step_count(
    httpserver: HTTPServer, monkeypatch: pytest.MonkeyPatch
) -> None:
    httpserver.expect_request("/v1/messages", method="POST").respond_with_json(
        {
            "model": "claude-3-5-sonnet-20241022",
            "content": [{"text": "ok"}],
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 4, "output_tokens": 2},
        }
    )
    monkeypatch.setenv("ANTHROPIC_API_KEY", "test-key")
    monkeypatch.setenv(
        "ARCFLOW_ANTHROPIC_API_ENDPOINT", httpserver.url_for("/v1/messages")
    )
    stub = Workflow("stub").step(_agent()).run("hello").step_count
    provider = (
        Workflow("anthropic")
        .step(_agent())
        .run("hello", provider=Anthropic(model="claude-3-5-sonnet-20241022"))
        .step_count
    )
    assert provider == stub == 1


def test_gemini_provider_swap_matches_stub_step_count(
    httpserver: HTTPServer, monkeypatch: pytest.MonkeyPatch
) -> None:
    httpserver.expect_request(
        "/v1beta/models/gemini-1.5-pro:generateContent", method="POST"
    ).respond_with_json(
        {
            "candidates": [
                {
                    "content": {"role": "model", "parts": [{"text": "ok"}]},
                    "finishReason": "STOP",
                }
            ],
            "usageMetadata": {
                "promptTokenCount": 2,
                "candidatesTokenCount": 3,
                "totalTokenCount": 5,
            },
        }
    )
    monkeypatch.setenv("GEMINI_API_KEY", "test-key")
    monkeypatch.setenv(
        "ARCFLOW_GEMINI_API_ENDPOINT", httpserver.url_for("/v1beta/models")
    )
    stub = Workflow("stub").step(_agent()).run("hello").step_count
    provider = (
        Workflow("gemini")
        .step(_agent())
        .run("hello", provider=Gemini(model="gemini-1.5-pro"))
        .step_count
    )
    assert provider == stub == 1


def test_stub_path_without_provider_unchanged() -> None:
    wf = Workflow("stub-only")
    wf.step(Agent(name="a", role="researcher", instructions="Summarize input."))
    result = wf.run("topic")
    assert result.step_count == 1
    assert result.output


def test_provider_trace_has_no_api_key(
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
        Workflow("redaction")
        .step(_agent())
        .run("hello", provider=OpenAI(model="gpt-4o"))
    )
    blob = get_execution_trace_json(result.run_id)
    assert secret not in blob
    parsed = json.loads(blob)
    assert isinstance(parsed, dict)
