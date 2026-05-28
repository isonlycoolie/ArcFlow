#!/usr/bin/env python3
"""Parse `cargo bench trace_overhead` output and fail if median exceeds budget."""

from __future__ import annotations

import re
import sys

# Sprint 5 budget: 3-step happy path under 500µs median (local dev gate).
MAX_MEDIAN_NS = 500_000


def main() -> int:
    text = sys.stdin.read()
    match = re.search(r"trace_emit_3_steps\s+([\d.]+)\s+ns", text)
    if not match:
        print("assert_trace_overhead: could not find bench line in stdin", file=sys.stderr)
        return 1
    median_ns = int(float(match.group(1)))
    if median_ns > MAX_MEDIAN_NS:
        print(
            f"trace overhead {median_ns}ns exceeds budget {MAX_MEDIAN_NS}ns",
            file=sys.stderr,
        )
        return 1
    print(f"trace overhead ok: {median_ns}ns <= {MAX_MEDIAN_NS}ns")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
