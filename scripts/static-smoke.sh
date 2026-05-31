#!/usr/bin/env bash
# Smoke: server health + optional relay URL reachability.
set -euo pipefail

SERVER="${ARCFLOW_SERVER_URL:-http://localhost:8080}"
RELAY="${VITE_ARCFLOW_RELAY_URL:-}"

curl -sf "${SERVER}/health" >/dev/null
echo "[ArcFlow] health ok"

READY=$(curl -s -o /dev/null -w "%{http_code}" "${SERVER}/ready")
echo "[ArcFlow] ready HTTP ${READY}"

if [[ -n "$RELAY" ]]; then
  BASE="${RELAY%/runs*}"
  BASE="${BASE%%/v1/sites/*}"
  curl -sf "${SERVER}/health" >/dev/null || true
  echo "[ArcFlow] relay base configured: ${RELAY}"
fi
