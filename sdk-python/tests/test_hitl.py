"""HITL SDK unit tests."""

from __future__ import annotations

import json

from arcflow import Agent, HitlConfig, Workflow


def test_step_hitl_serializes_to_rcs_payload() -> None:
    submit = Agent(name="submit", role="employee", instructions="submit expense")
    manager = Agent(name="manager", role="reviewer", instructions="review expense")
    wf = (
        Workflow("expense")
        .step(submit)
        .step(manager, hitl=HitlConfig(approval_key="manager_approval", timeout_seconds=3600))
    )
    payload = wf._build_run_payload("amount=100", '{"recovery_enabled": true}')  # noqa: SLF001
    steps = payload["workflow"]["steps"]  # type: ignore[index]
    assert len(steps) == 2
    assert steps[1]["hitl"]["approval_key"] == "manager_approval"
    assert steps[1]["hitl"]["timeout_seconds"] == 3600


def test_hitl_config_round_trip_json() -> None:
    cfg = HitlConfig(approval_key="key", timeout_seconds=120)
    parsed = json.loads(cfg.to_json())
    assert parsed["approval_key"] == "key"
    assert parsed["timeout_seconds"] == 120
    assert parsed["interrupt"] is True
