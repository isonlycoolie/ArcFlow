#!/usr/bin/env bash
# Publish default chat workflow. Usage: SITE_ID=... INSTRUCTIONS="..." ./static-publish-chat.sh
set -euo pipefail

SITE_ID="${SITE_ID:?SITE_ID required}"
INSTRUCTIONS="${INSTRUCTIONS:-Answer using the knowledge base. Be concise.}"
ADMIN_URL="${ARCFLOW_ADMIN_URL:-http://localhost:8080}"
ADMIN_KEY="${ARCFLOW_ADMIN_API_KEY:-dev-admin}"

BODY=$(INSTRUCTIONS="$INSTRUCTIONS" python -c "import json, os; print(json.dumps({'instructions': os.environ['INSTRUCTIONS']}))")
curl -sf -X POST "${ADMIN_URL}/v1/admin/sites/${SITE_ID}/workflows/chat/publish" \
  -H "X-ArcFlow-Admin-Key: ${ADMIN_KEY}" \
  -H "Content-Type: application/json" \
  -d "$BODY"
echo ""
echo "[ArcFlow] chat workflow published for site ${SITE_ID}"
