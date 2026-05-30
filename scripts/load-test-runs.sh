#!/usr/bin/env bash
# Load test POST /v1/runs (requires running server + Postgres migrations).
set -euo pipefail

BASE_URL="${ARCFLOW_SERVER_BASE_URL:-http://127.0.0.1:8080}"
API_KEY="${ARCFLOW_SERVER_API_KEY:-dev-secret}"
CONCURRENCY="${ARCFLOW_LOAD_CONCURRENCY:-100}"
MAX_P99_MS="${ARCFLOW_LOAD_MAX_P99_MS:-500}"

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

tmpdir="$(mktemp -d)"
trap 'rm -rf "${tmpdir}"' EXIT

run_one() {
  local start_ms end_ms elapsed code
  start_ms="$(python3 - <<'PY'
import time
print(int(time.time() * 1000))
PY
)"
  code="$(curl -s -o /dev/null -w "%{http_code}" -X POST "${BASE_URL}/v1/runs" \
    -H "Content-Type: application/json" \
    -H "X-ArcFlow-Api-Key: ${API_KEY}" \
    -d "${payload}" || echo "000")"
  end_ms="$(python3 - <<'PY'
import time
print(int(time.time() * 1000))
PY
)"
  elapsed=$((end_ms - start_ms))
  echo "${elapsed}" >> "${tmpdir}/times"
  echo "${code}" >> "${tmpdir}/codes"
}

echo "Starting ${CONCURRENCY} concurrent /v1/runs requests against ${BASE_URL}"
for _ in $(seq 1 "${CONCURRENCY}"); do
  run_one &
done
wait

python3 - "${tmpdir}" "${CONCURRENCY}" "${MAX_P99_MS}" <<'PY'
import sys
from pathlib import Path

times_path = Path(sys.argv[1]) / "times"
codes_path = Path(sys.argv[1]) / "codes"
expected = int(sys.argv[2])
max_p99 = int(sys.argv[3])

times = [int(line) for line in times_path.read_text().splitlines() if line.strip()]
codes = [line.strip() for line in codes_path.read_text().splitlines() if line.strip()]
success = sum(1 for code in codes if code.startswith("2"))
failed = len(codes) - success
times.sort()
p50 = times[len(times) // 2] if times else 0
p99 = times[max(0, int(len(times) * 0.99) - 1)] if times else 0

print(f"success={success}/{expected} failed={failed}")
print(f"p50_ms={p50} p99_ms={p99}")

if success != expected:
    raise SystemExit(f"load test failed: expected {expected} successes, got {success}")
if p99 > max_p99:
    raise SystemExit(f"load test failed: p99 {p99}ms exceeds budget {max_p99}ms")
print("load test passed")
PY
