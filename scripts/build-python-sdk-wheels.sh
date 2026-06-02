#!/usr/bin/env bash
# Build release wheels for sdk-python from repo root (cibuildwheel).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if ! command -v python3 >/dev/null 2>&1; then
  echo "ERROR: python3 required"
  exit 1
fi

python3 -m pip install -q "cibuildwheel>=2.16"
export CIBW_PROJECT_DIR="${ROOT}/sdk-python"
python3 -m cibuildwheel --output-dir "${ROOT}/sdk-python/dist" "${ROOT}/sdk-python"

echo "Wheels written to sdk-python/dist/"
ls -la "${ROOT}/sdk-python/dist/" || true
