#!/usr/bin/env bash
# Compile RCS JSON Schema (draft-07) — fails if schema is invalid.
# Requires Node.js and npx (CI and local dev).

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

npx --yes ajv-cli compile -s contracts/rcs-v1.schema.json --spec=draft7
echo "OK: contracts/rcs-v1.schema.json compiles as draft-07"
