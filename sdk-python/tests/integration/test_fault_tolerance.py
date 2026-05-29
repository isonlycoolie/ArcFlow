"""Sprint 7 chaos tests — see contracts/CHAOS-TEST-SPEC-v1.md."""

from __future__ import annotations

import json
import time

import pytest
from werkzeug.wrappers import Response

from arcflow import Agent, OpenAI, Workflow
from arcflow.exceptions import (
    ProviderExecutionError,
    RetryExhaustedError,
    WorkflowConfigurationError,
    WorkflowExecutionError,
    WorkflowTimeoutError,
)
from arcflow.retry import ConstantBackoff, ExponentialBackoff

pytest.importorskip("pytest_httpserver")
from pytest_httpserver import HTTPServer  # noqa: E402


def _agent() -> Agent:
    return Agent(name="writer", role="author", instructions="Write briefly.")


def _openai_ok(content: str = "ok") -> dict:
    return {
        "model": "gpt-4o",
        "choices": [{"message": {"content": content}, "finish_reason": "stop"}],
        "usage": {"prompt_tokens": 3, "completion_tokens": 2, "total_tokens": 5},
    }


def _openai_rate_limit() -> dict:
    return {"error": {"code": "rate_limit_exceeded", "type": "requests"}}


@pytest.fixture
def single_step_workflow() -> Workflow:
    wf = Workflow("fault_tolerance_test")
    wf.step(_agent())
    return wf


def test_provider_fails_first_call_succeeds_on_retry(
    httpserver: HTTPServer, monkeypatch: pytest.MonkeyPatch, single_step_workflow: Workflow
) -> None:
    calls = {"n": 0}

    def handler(_request):  # type: ignore[no-untyped-def]
        calls["n"] += 1
        if calls["n"] == 1:
            return Response(
                json.dumps(_openai_rate_limit()),
                status=429,
                mimetype="application/json",
            )
        return Response(
            json.dumps(_openai_ok()), status=200, mimetype="application/json"
        )

    httpserver.expect_request("/v1/chat/completions", method="POST").respond_with_handler(
        handler
    )
    monkeypatch.setenv("OPENAI_API_KEY", "test-key")
    monkeypatch.setenv(
        "ARCFLOW_OPENAI_API_ENDPOINT", httpserver.url_for("/v1/chat/completions")
    )
    single_step_workflow.retry(3, backoff=ConstantBackoff(delay_ms=1))
    result = single_step_workflow.run("input", provider=OpenAI(model="gpt-4o"))
    assert result.step_count == 1


def test_retry_exhausted_raises(single_step_workflow: Workflow, httpserver: HTTPServer, monkeypatch: pytest.MonkeyPatch) -> None:
    httpserver.expect_request("/v1/chat/completions", method="POST").respond_with_json(
        _openai_rate_limit(), status=429
    )
    monkeypatch.setenv("OPENAI_API_KEY", "test-key")
    monkeypatch.setenv(
        "ARCFLOW_OPENAI_API_ENDPOINT", httpserver.url_for("/v1/chat/completions")
    )
    single_step_workflow.retry(3, backoff=ConstantBackoff(delay_ms=1))
    with pytest.raises(RetryExhaustedError) as exc:
        single_step_workflow.run("input", provider=OpenAI(model="gpt-4o"))
    assert exc.value.attempts_made == 3
    assert "[ArcFlow]" in str(exc.value)


def test_non_retryable_error_fails_fast(
    httpserver: HTTPServer, monkeypatch: pytest.MonkeyPatch
) -> None:
    httpserver.expect_request("/v1/chat/completions", method="POST").respond_with_json(
        {"error": {"code": "invalid_api_key"}}, status=401
    )
    monkeypatch.setenv("OPENAI_API_KEY", "test-key")
    monkeypatch.setenv(
        "ARCFLOW_OPENAI_API_ENDPOINT", httpserver.url_for("/v1/chat/completions")
    )
    wf = Workflow("auth_fail")
    wf.step(_agent())
    wf.retry(3, backoff=ConstantBackoff(delay_ms=1000))
    start = time.time()
    with pytest.raises(ProviderExecutionError):
        wf.run("input", provider=OpenAI(model="gpt-4o"))
    assert time.time() - start < 0.5


def test_workflow_retry_zero_raises() -> None:
    wf = Workflow("cfg")
    wf.step(_agent())
    with pytest.raises(WorkflowConfigurationError, match=r"\[ArcFlow\]"):
        wf.retry(0)


def test_exponential_backoff_invalid_multiplier() -> None:
    with pytest.raises(WorkflowConfigurationError, match=r"\[ArcFlow\]"):
        ExponentialBackoff(base_ms=100, multiplier=0.5)


def test_retry_chaining() -> None:
    wf = Workflow("chain")
    wf.step(_agent())
    assert wf.retry(3) is wf


STUB_FAIL_ROLE = "__fail__"


def _event_kinds(events: tuple[dict, ...]) -> set[str]:
    return {str(e.get("event_kind", "")) for e in events}


def test_exponential_backoff_timing(
    httpserver: HTTPServer, monkeypatch: pytest.MonkeyPatch
) -> None:
    calls = {"n": 0}

    def handler(_request):  # type: ignore[no-untyped-def]
        calls["n"] += 1
        return Response(
            json.dumps(_openai_rate_limit()),
            status=429,
            mimetype="application/json",
        )

    httpserver.expect_request("/v1/chat/completions", method="POST").respond_with_handler(
        handler
    )
    monkeypatch.setenv("OPENAI_API_KEY", "test-key")
    monkeypatch.setenv(
        "ARCFLOW_OPENAI_API_ENDPOINT", httpserver.url_for("/v1/chat/completions")
    )
    wf = Workflow("backoff_timing")
    wf.step(_agent())
    wf.retry(4, backoff=ExponentialBackoff(base_ms=100, multiplier=2.0, max_ms=500))
    start = time.time()
    with pytest.raises(RetryExhaustedError):
        wf.run("input", provider=OpenAI(model="gpt-4o"))
    elapsed = time.time() - start
    assert elapsed >= 0.25
    assert calls["n"] == 4


def test_step_fallback_activates_trace_event() -> None:
    fallback = Agent(name="backup", role="backup", instructions="Fallback path.")
    primary = Agent(name="primary", role=STUB_FAIL_ROLE, instructions="Fails.")
    wf = Workflow("fallback_flow")
    wf.step(fallback)
    wf.step(primary, fallback=fallback)
    result = wf.run("input")
    assert result.step_count == 2
    assert "backup" in result.output


def test_step_timeout_enforced(
    httpserver: HTTPServer, monkeypatch: pytest.MonkeyPatch
) -> None:
    def slow_handler(_request):  # type: ignore[no-untyped-def]
        time.sleep(2)
        return Response(
            json.dumps(_openai_ok()), status=200, mimetype="application/json"
        )

    httpserver.expect_request("/v1/chat/completions", method="POST").respond_with_handler(
        slow_handler
    )
    monkeypatch.setenv("OPENAI_API_KEY", "test-key")
    monkeypatch.setenv(
        "ARCFLOW_OPENAI_API_ENDPOINT", httpserver.url_for("/v1/chat/completions")
    )
    wf = Workflow("step_timeout")
    wf.step(_agent())
    wf.step_timeout(1)
    with pytest.raises(WorkflowTimeoutError) as exc:
        wf.run("input", provider=OpenAI(model="gpt-4o"))
    assert exc.value.timeout_type == "step"


def test_workflow_timeout_enforced(
    httpserver: HTTPServer, monkeypatch: pytest.MonkeyPatch
) -> None:
    def slow_handler(_request):  # type: ignore[no-untyped-def]
        time.sleep(2)
        return Response(
            json.dumps(_openai_ok()), status=200, mimetype="application/json"
        )

    httpserver.expect_request("/v1/chat/completions", method="POST").respond_with_handler(
        slow_handler
    )
    monkeypatch.setenv("OPENAI_API_KEY", "test-key")
    monkeypatch.setenv(
        "ARCFLOW_OPENAI_API_ENDPOINT", httpserver.url_for("/v1/chat/completions")
    )
    wf = Workflow("workflow_timeout")
    wf.step(_agent())
    wf.timeout(1)
    with pytest.raises(WorkflowTimeoutError) as exc:
        wf.run("input", provider=OpenAI(model="gpt-4o"))
    assert exc.value.timeout_type == "workflow"


@pytest.mark.skipif(
    not __import__("os").environ.get("ARCFLOW_POSTGRESQL_URL"),
    reason="recovery tests require ARCFLOW_POSTGRESQL_URL",
)
def test_partial_recovery_resume() -> None:
    middle = Agent(name="mid", role=STUB_FAIL_ROLE, instructions="fail mid")
    ok = Agent(name="ok", role="writer", instructions="ok")
    wf = Workflow("recovery_flow")
    wf.step(ok)
    wf.step(middle)
    wf.step(ok)
    wf.enable_recovery()
    with pytest.raises(WorkflowExecutionError) as exc:
        wf.run("recovery-input")
    run_id = exc.value.run_id
    assert run_id
    resumed = wf.resume(run_id)
    # One completed step before failure + final step after resume (failed step skipped).
    assert resumed.step_count == 2
    assert "writer" in resumed.output
