#!/usr/bin/env python3
"""Fail CI when provider-related files contain credential-like patterns."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCAN_DIRS = (
    ROOT / "runtime" / "arcflow-core" / "src" / "providers",
    ROOT / "sdk-python" / "arcflow",
    ROOT / "sdk-typescript" / "arcflow",
)

PATTERNS = (
    re.compile(r"sk-[A-Za-z0-9]{20,}"),
    re.compile(r"AKIA[0-9A-Z]{16}"),
    re.compile(r"BEGIN (RSA|OPENSSH|EC) PRIVATE KEY"),
)


def main() -> int:
    violations: list[str] = []
    for base in SCAN_DIRS:
        if not base.exists():
            continue
        for path in base.rglob("*"):
            if path.suffix not in {".rs", ".py", ".ts", ".js"}:
                continue
            text = path.read_text(encoding="utf-8")
            for pattern in PATTERNS:
                for match in pattern.finditer(text):
                    violations.append(f"{path}:{match.group()}")
    if violations:
        print("Provider credential pattern violations:")
        for line in violations:
            print(f"  {line}")
        return 1
    print("OK: no provider credential patterns in SDK/provider sources")
    return 0


if __name__ == "__main__":
    sys.exit(main())
