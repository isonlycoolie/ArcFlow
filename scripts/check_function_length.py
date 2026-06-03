#!/usr/bin/env python3
"""Fail when arcflow-core functions exceed MAX lines (brace-accurate, fast)."""

from __future__ import annotations

import re
import sys
from pathlib import Path

MAX_FUNCTION_LINES = 40
SRC = Path("runtime/arcflow-core/src")
ALLOWLIST = Path("scripts/function-length-allowlist.txt")

FN_RE = re.compile(r"(?:^|\s)(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+(\w+)\s*\(")


def strip_tests(text: str) -> str:
    marker = "#[cfg(test)]"
    return text.split(marker, 1)[0] if marker in text else text


def function_span(text: str, fn_start: int) -> tuple[int, int] | None:
    brace = -1
    i = fn_start
    while i < len(text):
        ch = text[i]
        if ch == "{":
            if brace < 0:
                brace = 0
            brace += 1
        elif ch == "}":
            if brace > 0:
                brace -= 1
                if brace == 0:
                    start_line = text.count("\n", 0, fn_start) + 1
                    end_line = text.count("\n", 0, i) + 1
                    return start_line, end_line
        i += 1
    return None


def load_allowlist() -> set[tuple[str, str]]:
    if not ALLOWLIST.is_file():
        return set()
    out: set[tuple[str, str]] = set()
    for line in ALLOWLIST.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        path, _, name = line.partition(":")
        out.add((path.replace("\\", "/"), name))
    return out


def list_overlimit() -> int:
    allow = load_allowlist()
    for path in sorted(SRC.rglob("*.rs")):
        rel = path.as_posix()
        text = strip_tests(path.read_text(encoding="utf-8"))
        for match in FN_RE.finditer(text):
            name = match.group(1)
            span = function_span(text, match.start())
            if span is None:
                continue
            lines = span[1] - span[0] + 1
            if lines > MAX_FUNCTION_LINES and (rel, name) not in allow:
                print(f"{rel}:{name}  # {lines} lines")
    return 0


def main() -> int:
    if len(sys.argv) > 1 and sys.argv[1] == "--list-overlimit":
        return list_overlimit()
    allow = load_allowlist()
    failed: list[str] = []
    for path in sorted(SRC.rglob("*.rs")):
        rel = path.as_posix()
        text = strip_tests(path.read_text(encoding="utf-8"))
        for match in FN_RE.finditer(text):
            name = match.group(1)
            if (rel, name) in allow:
                continue
            span = function_span(text, match.start())
            if span is None:
                continue
            start_line, end_line = span
            lines = end_line - start_line + 1
            if lines > MAX_FUNCTION_LINES:
                failed.append(f"ERROR: {rel} fn {name} is {lines} lines (max {MAX_FUNCTION_LINES})")
    if failed:
        print("\n".join(sorted(failed)), file=sys.stderr)
        return 1
    print(f"OK: all functions within {MAX_FUNCTION_LINES} lines (allowlist: {len(allow)} entries)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
