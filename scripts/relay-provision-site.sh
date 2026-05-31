#!/usr/bin/env bash
# Provision a Relay site via admin API. Requires arcflow-server with Postgres.
set -euo pipefail

ADMIN_URL="${ARCFLOW_ADMIN_URL:-http://localhost:8080}"
ADMIN_KEY="${ARCFLOW_ADMIN_API_KEY:-dev-admin}"
ORIGIN="${ARCFLOW_SITE_ORIGIN:-http://localhost:5173}"
NAME="${ARCFLOW_SITE_NAME:-Local Dev Site}"

RESP=$(curl -sf -X POST "${ADMIN_URL}/v1/admin/sites" \
  -H "X-ArcFlow-Admin-Key: ${ADMIN_KEY}" \
  -H "Content-Type: application/json" \
  -d "{\"display_name\":\"${NAME}\",\"allowed_origins\":[\"${ORIGIN}\"]}")

SITE_ID=$(echo "$RESP" | sed -n 's/.*"site_id"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
RELAY_URL=$(echo "$RESP" | sed -n 's/.*"relay_url"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
TOKEN=$(echo "$RESP" | sed -n 's/.*"site_token"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')

echo "# ArcFlow Relay site provisioned"
echo "SITE_ID=${SITE_ID}"
echo ""
echo "# Add to your static site .env:"
echo "VITE_ARCFLOW_RELAY_URL=${RELAY_URL}"
echo "VITE_ARCFLOW_SITE_TOKEN=${TOKEN}"
