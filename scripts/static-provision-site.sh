#!/usr/bin/env bash
# Create a Relay site via admin API. Prints Vite env vars.
set -euo pipefail

ADMIN_URL="${ARCFLOW_ADMIN_URL:-http://localhost:8080}"
ADMIN_KEY="${ARCFLOW_ADMIN_API_KEY:-dev-admin}"
ORIGIN="${ARCFLOW_SITE_ORIGIN:-http://localhost:5173}"
NAME="${ARCFLOW_SITE_NAME:-Static Dev Site}"

RESP=$(curl -sf -X POST "${ADMIN_URL}/v1/admin/sites" \
  -H "X-ArcFlow-Admin-Key: ${ADMIN_KEY}" \
  -H "Content-Type: application/json" \
  -d "{\"display_name\":\"${NAME}\",\"allowed_origins\":[\"${ORIGIN}\"]}")

if command -v jq >/dev/null 2>&1; then
  SITE_ID=$(echo "$RESP" | jq -r '.site_id')
  RELAY_URL=$(echo "$RESP" | jq -r '.relay_url')
  TOKEN=$(echo "$RESP" | jq -r '.site_token')
else
  SITE_ID=$(echo "$RESP" | sed -n 's/.*"site_id"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
  RELAY_URL=$(echo "$RESP" | sed -n 's/.*"relay_url"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
  TOKEN=$(echo "$RESP" | sed -n 's/.*"site_token"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
fi

echo "SITE_ID=${SITE_ID}"
echo "VITE_ARCFLOW_RELAY_URL=${RELAY_URL}"
echo "VITE_ARCFLOW_SITE_TOKEN=${TOKEN}"
