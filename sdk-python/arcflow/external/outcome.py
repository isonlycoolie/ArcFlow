"""HTTP client for external outcome callbacks."""

from __future__ import annotations

import hashlib
import hmac
import json
import os
from typing import Any
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen


def _sign_body(secret: str, body: bytes) -> str:
    digest = hmac.new(secret.encode(), body, hashlib.sha256).hexdigest()
    return f"sha256={digest}"


def report_outcome(
    run_id: str,
    binding_id: str,
    outcome: dict[str, Any],
    *,
    base_url: str = "http://localhost:8080",
    api_key: str | None = None,
    webhook_secret: str | None = None,
    idempotency_key: str | None = None,
) -> dict[str, Any]:
    """POST an ExternalOutcomeReport to the ArcFlow server callback endpoint."""
    api_key = api_key or os.environ.get("ARCFLOW_SERVER_API_KEY", "")
    webhook_secret = webhook_secret or os.environ.get("ARCFLOW_WEBHOOK_SECRET", "")
    if not api_key:
        raise ValueError("[ArcFlow] ARCFLOW_SERVER_API_KEY is required")
    if not webhook_secret:
        raise ValueError("[ArcFlow] ARCFLOW_WEBHOOK_SECRET is required")

    payload = {"binding_id": binding_id, **outcome}
    body = json.dumps(payload).encode()
    headers = {
        "Content-Type": "application/json",
        "Authorization": f"Bearer {api_key}",
        "X-ArcFlow-Signature": _sign_body(webhook_secret, body),
    }
    if idempotency_key:
        headers["X-Idempotency-Key"] = idempotency_key

    url = f"{base_url.rstrip('/')}/v1/runs/{run_id}/external/{binding_id}"
    req = Request(url, data=body, headers=headers, method="POST")
    try:
        with urlopen(req, timeout=30) as resp:
            return json.loads(resp.read().decode())
    except HTTPError as e:
        detail = e.read().decode() if e.fp else str(e)
        raise RuntimeError(f"[ArcFlow] external callback failed ({e.code}): {detail}") from e
    except URLError as e:
        raise RuntimeError(f"[ArcFlow] external callback network error: {e}") from e
