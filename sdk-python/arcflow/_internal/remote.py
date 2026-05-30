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
