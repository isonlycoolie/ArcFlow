#!/usr/bin/env bash
# Approve a pending expense run via the ArcFlow HTTP server.
set -euo pipefail

RUN_ID="${1:?usage: approve_cli.sh RUN_ID}"
BASE_URL="${ARCFLOW_RUNTIME:-http://localhost:8080}"
API_KEY="${ARCFLOW_SERVER_API_KEY:?set ARCFLOW_SERVER_API_KEY}"

curl -sS -X POST "${BASE_URL}/v1/runs/${RUN_ID}/approve/manager_approval" \
  -H "Content-Type: application/json" \
  -H "X-ArcFlow-Api-Key: ${API_KEY}" \
  -d '{"approved": true, "data": {"manager_id": "mgr-42"}}'

echo ""
curl -sS "${BASE_URL}/v1/runs/${RUN_ID}" \
  -H "X-ArcFlow-Api-Key: ${API_KEY}"
echo ""
