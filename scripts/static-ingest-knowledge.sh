#!/usr/bin/env bash
# Ingest knowledge for a site. Usage: SITE_ID=... TEXT_FILE=kb.txt ./static-ingest-knowledge.sh
set -euo pipefail

SITE_ID="${SITE_ID:?SITE_ID required}"
TEXT_FILE="${TEXT_FILE:?TEXT_FILE required}"
ADMIN_URL="${ARCFLOW_ADMIN_URL:-http://localhost:8080}"
ADMIN_KEY="${ARCFLOW_ADMIN_API_KEY:-dev-admin}"
KEY="${ARCFLOW_KB_KEY:-faq}"

JSON=$(python -c "import json, pathlib; print(json.dumps({'text': pathlib.Path('${TEXT_FILE}').read_text(), 'key': '${KEY}'}))")

curl -sf -X POST "${ADMIN_URL}/v1/admin/sites/${SITE_ID}/knowledge/ingest" \
  -H "X-ArcFlow-Admin-Key: ${ADMIN_KEY}" \
  -H "Content-Type: application/json" \
  -d "$JSON"
echo ""
echo "[ArcFlow] knowledge ingested for site ${SITE_ID}"
