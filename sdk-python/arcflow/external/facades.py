"""PascalCase facades for external binding callbacks."""

from __future__ import annotations

from typing import Any

from arcflow.external.outcome import report_outcome


class ExternalOutcome:
    """Post signed external binding outcomes to the ArcFlow server."""

    @staticmethod
    def report(
        run_id: str,
        binding_id: str,
        outcome: dict[str, Any],
        *,
        base_url: str = "http://localhost:8080",
        api_key: str | None = None,
        webhook_secret: str | None = None,
        idempotency_key: str | None = None,
    ) -> dict[str, Any]:
        return report_outcome(
            run_id,
            binding_id,
            outcome,
            base_url=base_url,
            api_key=api_key,
            webhook_secret=webhook_secret,
            idempotency_key=idempotency_key,
        )
