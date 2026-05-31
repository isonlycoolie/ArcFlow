#!/usr/bin/env python3
# Post external binding outcome — simulates a government portal or RPA worker callback.

from __future__ import annotations

import argparse
import os
import sys

from arcflow.external import ExternalOutcome


def main() -> int:
    parser = argparse.ArgumentParser(description="Post external binding outcome")
    parser.add_argument("--run-id", required=True)
    parser.add_argument("--binding-id", default="gov_portal_submit")
    parser.add_argument("--status", default="needs_input", choices=["success", "failed", "needs_input"])
    parser.add_argument("--error-code", default="INVALID_NAME")
    parser.add_argument("--base-url", default=os.environ.get("ARCFLOW_BASE_URL", "http://localhost:8080"))
    args = parser.parse_args()

    outcome = {"status": args.status}
    if args.error_code:
        outcome["error_code"] = args.error_code

    try:
        resp = ExternalOutcome.report(
            args.run_id,
            args.binding_id,
            outcome,
            base_url=args.base_url,
        )
        print(resp)
        return 0
    except Exception as e:
        print(f"[ArcFlow] callback failed: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
