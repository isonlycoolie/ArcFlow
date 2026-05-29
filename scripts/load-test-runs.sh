#!/usr/bin/env bash
# Load test POST /v1/runs (requires running server + Postgres migrations).
set -euo pipefail

BASE_URL="${ARCFLOW_SERVER_BASE_URL:-http://127.0.0.1:8080}"
API_KEY="${ARCFLOW_SERVER_API_KEY:-dev-secret}"
CONCURRENCY="${ARCFLOW_LOAD_CONCURRENCY:-100}"

payload='{
  "workflow": {
    "id": "00000000-0000-4000-8000-000000000099",
    "name": "load_test",
    "execution_mode": "linear",
    "steps": [
      {
        "id": "00000000-0000-4000-8000-000000000001",
        "agent_id": "00000000-0000-4000-8000-000000000010",
        "order": 1
      }
    ]
  },
  "agents": [
    {
      "id": "00000000-0000-4000-8000-000000000010",
      "name": "writer",
      "role": "author",
      "instructions": "Reply briefly."
    }
  ],
  "input": "load test"
}'

run_one() {
  curl -sf -X POST "${BASE_URL}/v1/runs" \
    -H "Content-Type: application/json" \
    -H "X-ArcFlow-Api-Key: ${API_KEY}" \
    -d "${payload}" >/dev/null
}

echo "Starting ${CONCURRENCY} concurrent /v1/runs requests against ${BASE_URL}"
for _ in $(seq 1 "${CONCURRENCY}"); do
  run_one &
done
wait
echo "Load test completed"
