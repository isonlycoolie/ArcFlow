"""Helpers for building workflow.test() cases (Phase 2.3)."""

from __future__ import annotations

from typing import Any


def normalize_test_case(case: dict[str, Any]) -> dict[str, Any]:
    """Maps spec aliases ``mock_step_failure`` / ``mock_fail_count`` to stub_responses."""
    normalized = dict(case)
    mock_step = normalized.get("mock_step_failure")
    mock_fail_count = normalized.get("mock_fail_count")
    if mock_step is not None:
        key = str(mock_step)
        fail_n = int(mock_fail_count) if mock_fail_count is not None else 1
        stub = dict(normalized.get("stub_responses") or {})
        entry: dict[str, Any] = {"fail_times": fail_n}
        if "expected_output" in normalized:
            entry["then_output"] = normalized["expected_output"]
        stub[key] = {**stub.get(key, {}), **entry}
        normalized["stub_responses"] = stub
    return normalized


def assert_retries_met(
    case: dict[str, Any], attempts_made: int | None, passed: bool
) -> bool:
    expected = case.get("assert_retries")
    if expected is None:
        return passed
    if attempts_made is None:
        return False
    return passed and attempts_made == int(expected)
