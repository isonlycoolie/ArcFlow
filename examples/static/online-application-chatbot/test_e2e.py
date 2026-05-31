"""E2E validation for online application chatbot reference agent."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from arcflow.external import ExternalBindingConfig, report_outcome
from arcflow.schedule import ScheduleManifest


ROOT = Path(__file__).resolve().parent


def test_schedule_manifest_validates():
    manifest = ScheduleManifest.load(ROOT / "arcflow.schedule.yaml")
    manifest.validate()


def test_external_binding_config_serializes():
    cfg = ExternalBindingConfig(
        "gov_portal_submit",
        attach_to_step_id="550e8400-e29b-41d4-a716-446655440000",
    )
    data = cfg.to_dict()
    assert data["id"] == "gov_portal_submit"
    assert data["mode"] == "async_callback"


def test_sample_run_json_shape():
    payload = json.loads((ROOT / "sample_run.json").read_text(encoding="utf-8"))
    assert "initial_state" in payload
    assert "conversation_turns" in payload["initial_state"]


@pytest.mark.skipif(
    not __import__("os").environ.get("ARCFLOW_E2E"),
    reason="Set ARCFLOW_E2E=1 with server running for live callback test",
)
def test_report_outcome_live():
    run_id = __import__("os").environ["ARCFLOW_E2E_RUN_ID"]
    resp = report_outcome(
        run_id,
        "gov_portal_submit",
        {"status": "needs_input", "error_code": "INVALID_NAME"},
    )
    assert "run_id" in resp
