"""Serialize Sprint 7 execution options for the native binding."""

from __future__ import annotations

import json
from typing import Any

from arcflow.retry import BackoffStrategy, ConstantBackoff, ExponentialBackoff, LinearBackoff


def backoff_to_dict(backoff: BackoffStrategy) -> dict[str, Any]:
    if isinstance(backoff, ExponentialBackoff):
        return {
            "kind": "exponential",
            "base_ms": backoff.base_ms,
            "multiplier": backoff.multiplier,
            "max_ms": backoff.max_ms,
            "jitter_ms": backoff.jitter_ms,
        }
    if isinstance(backoff, LinearBackoff):
        return {
            "kind": "linear",
            "base_ms": backoff.base_ms,
            "increment_ms": backoff.increment_ms,
            "max_ms": backoff.max_ms,
            "jitter_ms": backoff.jitter_ms,
        }
    if isinstance(backoff, ConstantBackoff):
        return {
            "kind": "constant",
            "delay_ms": backoff.delay_ms,
            "jitter_ms": backoff.jitter_ms,
        }
    raise TypeError(f"Unknown backoff type: {type(backoff)!r}")


def build_exec_config_json(
    *,
    retry: tuple[int, BackoffStrategy] | None,
    workflow_timeout_seconds: float | None,
    step_timeout_seconds: float | None,
    recovery_enabled: bool,
    test: dict[str, Any] | None = None,
) -> str | None:
    payload: dict[str, Any] = {}
    if retry is not None:
        max_attempts, backoff = retry
        payload["retry"] = {
            "max_attempts": max_attempts,
            "backoff": backoff_to_dict(backoff),
        }
    if workflow_timeout_seconds is not None:
        payload["workflow_timeout_secs"] = workflow_timeout_seconds
    if step_timeout_seconds is not None:
        payload["step_timeout_secs"] = step_timeout_seconds
    if recovery_enabled:
        payload["recovery_enabled"] = True
    if test is not None:
        payload["test"] = test
    if not payload:
        return None
    return json.dumps(payload)
