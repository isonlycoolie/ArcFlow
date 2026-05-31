"""External binding configuration (Phase 2-Pro v2)."""

from __future__ import annotations

from typing import Any


class ExternalBindingConfig:
    """Declarative external binding metadata for workflow publish payloads."""

    def __init__(
        self,
        binding_id: str,
        *,
        kind: str = "browser_automation",
        attach_to_step_id: str,
        mode: str = "async_callback",
        outcome_schema: dict[str, Any] | None = None,
        recovery: dict[str, Any] | None = None,
    ) -> None:
        self.binding_id = binding_id
        self.kind = kind
        self.attach_to_step_id = attach_to_step_id
        self.mode = mode
        self.outcome_schema = outcome_schema or {
            "type": "object",
            "properties": {
                "status": {"enum": ["success", "failed", "needs_input"]},
            },
            "required": ["status"],
        }
        self.recovery = recovery

    def to_dict(self) -> dict[str, Any]:
        payload: dict[str, Any] = {
            "id": self.binding_id,
            "kind": self.kind,
            "attach_to_step_id": self.attach_to_step_id,
            "mode": self.mode,
            "outcome_schema": self.outcome_schema,
        }
        if self.recovery:
            payload["recovery"] = self.recovery
        return payload
