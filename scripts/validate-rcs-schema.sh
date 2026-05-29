#!/usr/bin/env bash
# Compile RCS JSON Schema (draft-07) — fails if schema is invalid.
# Requires Node.js and npx (CI and local dev).
#
# Note: `ajv validate` without `-d` is not applicable — this file is $defs-only
# (no root schema document). Use `ajv compile` here; instance validation is a
# separate step when a concrete root + data file exist.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

npx --yes ajv-cli compile -s contracts/normative/rcs/v1.schema.json --spec=draft7
echo "OK: contracts/normative/rcs/v1.schema.json compiles as draft-07"
