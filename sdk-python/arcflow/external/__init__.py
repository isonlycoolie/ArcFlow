"""External binding helpers and outcome reporting (Phase 2-Pro v2)."""

from __future__ import annotations

import warnings

from arcflow.external.binding import ExternalBindingConfig
from arcflow.external.facades import ExternalOutcome
from arcflow.external.outcome import report_outcome as _report_outcome


def report_outcome(*args: object, **kwargs: object) -> dict[str, object]:
    warnings.warn(
        "report_outcome is deprecated; use ExternalOutcome.report(...)",
        DeprecationWarning,
        stacklevel=2,
    )
    return _report_outcome(*args, **kwargs)  # type: ignore[arg-type]


__all__ = [
    "ExternalBindingConfig",
    "ExternalOutcome",
    "report_outcome",
]
