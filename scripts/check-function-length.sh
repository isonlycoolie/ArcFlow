#!/usr/bin/env bash
# Fail when any function in arcflow-core/src exceeds MAX_FUNCTION_LINES (Sprint 2: 40).

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

python3 scripts/check_function_length.py
