"""Unit tests for arcflow.external facades."""

from __future__ import annotations

import warnings

import pytest

from arcflow.external import ExternalBindingConfig, ExternalOutcome


def test_external_binding_config_serializes() -> None:
    cfg = ExternalBindingConfig(
        "gov_portal_submit",
        attach_to_step_id="550e8400-e29b-41d4-a716-446655440000",
    )
    data = cfg.to_dict()
    assert data["id"] == "gov_portal_submit"
    assert data["mode"] == "async_callback"


def test_canonical_import_no_deprecation_warning() -> None:
    with warnings.catch_warnings():
        warnings.simplefilter("error", DeprecationWarning)
        assert ExternalOutcome.report is not None


def test_deprecated_report_outcome_warns() -> None:
    from arcflow.external import report_outcome

    with pytest.raises(ValueError, match=r"ARCFLOW_SERVER_API_KEY"):
        with warnings.catch_warnings(record=True) as caught:
            warnings.simplefilter("always")
            report_outcome("run-1", "binding-1", {"status": "success"})
    assert any(
        issubclass(w.category, DeprecationWarning) for w in caught
    )
