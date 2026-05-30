"""HTTP client for remote ArcFlow server execution."""

from __future__ import annotations

import json
import os
import urllib.error
import urllib.request
from typing import TYPE_CHECKING, Any

from arcflow.exceptions import WorkflowConfigurationError, WorkflowExecutionError
from arcflow.hitl import WorkflowInterruptedError
from arcflow.result import WorkflowResult

if TYPE_CHECKING:
    from arcflow.workflow import Workflow

API_KEY_HEADER = "X-ArcFlow-Api-Key"


def run_workflow(
    workflow: Workflow,
    run_input: str,
    *,
    exec_config_json: str | None = None,
) -> WorkflowResult:
    base_url = workflow._runtime_url  # noqa: SLF001
    if not base_url:
        raise WorkflowConfigurationError(
            "[ArcFlow] Remote runtime URL is not configured."
        )
    payload = workflow._build_run_payload(run_input, exec_config_json)  # noqa: SLF001
    response = _post_json(f"{base_url}/v1/runs", payload)
    status = response.get("status", "")
    run_id = str(response.get("run_id", ""))
    if status.lower() == "completed":
        detail = _get_json(f"{base_url}/v1/runs/{run_id}")
        result = detail.get("result") or {}
        return WorkflowResult(
            output=str(result.get("output", "")),
            run_id=run_id,
            step_count=int(result.get("step_count", 0)),
            trace_events=(),
            status="completed",
        )
    if status.lower() == "interrupted":
        detail = _get_json(f"{base_url}/v1/runs/{run_id}")
        interrupt = detail.get("interrupt") or {}
        raise WorkflowInterruptedError(
            f"[ArcFlow] Workflow run '{run_id}' paused for human approval.",
            run_id=run_id,
            approval_key=str(interrupt.get("approval_key", "")),
            expires_at=str(interrupt.get("expires_at", "")) or None,
        )
    if status.lower() == "failed":
        detail = _get_json(f"{base_url}/v1/runs/{run_id}")
        err = detail.get("error") or {}
        raise WorkflowExecutionError(
            str(err.get("message", "remote run failed")),
            run_id=run_id,
            failed_step=err.get("step_id"),
        )
    return WorkflowResult(
        output="",
        run_id=run_id,
        step_count=0,
        trace_events=(),
        status=status.lower(),
    )


def approve_run(
    workflow: Workflow,
    run_id: str,
    approval_key: str,
    *,
    approved: bool = True,
    data: dict[str, object] | None = None,
) -> WorkflowResult:
    base_url = workflow._runtime_url  # noqa: SLF001
    if not base_url:
        raise WorkflowConfigurationError(
            "[ArcFlow] Remote runtime URL is not configured."
        )
    payload = {"approved": approved, "data": data or {}}
    response = _post_json(
        f"{base_url}/v1/runs/{run_id}/approve/{approval_key}",
        payload,
    )
    final_status = str(response.get("status", "running")).lower()
    if final_status == "completed":
        detail = _get_json(f"{base_url}/v1/runs/{run_id}")
        result = detail.get("result") or {}
        return WorkflowResult(
            output=str(result.get("output", "")),
            run_id=run_id,
            step_count=int(result.get("step_count", 0)),
            trace_events=(),
            status="completed",
        )
    detail = _get_json(f"{base_url}/v1/runs/{run_id}")
    if str(detail.get("status", "")).lower() == "failed":
        err = detail.get("error") or {}
        raise WorkflowExecutionError(
            str(err.get("message", "remote run failed after approval")),
            run_id=run_id,
            failed_step=err.get("step_id"),
        )
    return WorkflowResult(
        output="",
        run_id=run_id,
        step_count=0,
        trace_events=(),
        status=final_status,
    )


def _api_key() -> str | None:
    return os.environ.get("ARCFLOW_SERVER_API_KEY")


def _post_json(url: str, payload: dict[str, Any]) -> dict[str, Any]:
    return _request("POST", url, payload)


def _get_json(url: str) -> dict[str, Any]:
    return _request("GET", url, None)


def _request(
    method: str, url: str, payload: dict[str, Any] | None
) -> dict[str, Any]:
    data = json.dumps(payload).encode("utf-8") if payload is not None else None
    req = urllib.request.Request(url, data=data, method=method)
    req.add_header("Content-Type", "application/json")
    key = _api_key()
    if key:
        req.add_header(API_KEY_HEADER, key)
    try:
        with urllib.request.urlopen(req, timeout=120) as resp:
            body = resp.read().decode("utf-8")
            return json.loads(body) if body else {}
    except urllib.error.HTTPError as exc:
        detail = exc.read().decode("utf-8", errors="replace")
        raise WorkflowConfigurationError(
            f"[ArcFlow] Remote request failed ({exc.code}): {detail}"
        ) from exc
    except urllib.error.URLError as exc:
        raise WorkflowConfigurationError(
            f"[ArcFlow] Could not reach remote runtime: {exc.reason}"
        ) from exc
