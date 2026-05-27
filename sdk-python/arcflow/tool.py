"""Tool — external capability invoked by the runtime."""

from __future__ import annotations

import json
from collections.abc import Callable
from typing import Any

from arcflow.exceptions import ToolConfigurationError


def _require_non_empty(field: str, value: str) -> str:
    trimmed = value.strip()
    if not trimmed:
        raise ToolConfigurationError(
            f"[ArcFlow] Tool {field} must be a non-empty string."
        )
    return trimmed


class Tool:
    """Declares a named capability with JSON Schema input validation."""

    def __init__(
        self,
        name: str,
        description: str,
        input_schema: dict[str, Any],
        execute: Callable[[dict[str, Any]], str],
        timeout_seconds: float = 30.0,
    ) -> None:
        self.name = _require_non_empty("name", name)
        self.description = _require_non_empty("description", description)
        if not isinstance(input_schema, dict):
            raise ToolConfigurationError(
                "[ArcFlow] Tool input_schema must be a dict (JSON Schema object)."
            )
        try:
            json.dumps(input_schema)
        except (TypeError, ValueError) as exc:
            raise ToolConfigurationError(
                "[ArcFlow] Tool input_schema must be JSON-serializable."
            ) from exc
        if not callable(execute):
            raise ToolConfigurationError(
                "[ArcFlow] Tool execute must be a callable accepting one dict."
            )
        if timeout_seconds <= 0:
            raise ToolConfigurationError(
                "[ArcFlow] Tool timeout_seconds must be positive."
            )
        self.input_schema = input_schema
        self.execute = execute
        self.timeout_seconds = timeout_seconds

    def binding_spec(self) -> tuple[str, str, str, float]:
        """Serializes tool metadata for the native binding (no callable)."""
        return (
            self.name,
            self.description,
            json.dumps(self.input_schema),
            self.timeout_seconds,
        )
