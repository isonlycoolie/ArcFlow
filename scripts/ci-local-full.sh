#!/usr/bin/env bash
# Full CI: ci-local.sh plus ci-full.yml jobs. See scripts/README.md for skip env vars.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
export CARGO_TERM_COLOR=always RUST_BACKTRACE=1
run_step() { echo ""; echo "=== $1 ==="; shift; "$@"; }
bash scripts/ci-local.sh
run_step "trace-performance" bash -c '
  cargo bench -p arcflow-core --bench trace_overhead -- --noplot 2>&1 | tee /tmp/arcflow-trace-bench.txt
  python scripts/assert_trace_overhead.py < /tmp/arcflow-trace-bench.txt
'
command -v cargo-audit &>/dev/null || cargo install cargo-audit
run_step "audit" cargo audit
run_step "doc" env RUSTDOCFLAGS=-D warnings cargo doc --workspace --no-deps
run_step "typescript-build" bash -c '
  cargo build -p arcflow-node
  cd sdk-typescript && npm ci && npm run build:ts && npm test
'
if [[ "${CI_SKIP_POSTGRES:-}" == "1" || -z "${ARCFLOW_POSTGRESQL_URL:-}" ]]; then
  echo ""; echo "=== postgres-integration (skipped) ==="
else
  run_step "postgres-integration" bash -c '
    for f in runtime/arcflow-core/migrations/*.sql; do psql "$ARCFLOW_POSTGRESQL_URL" -f "$f"; done
    cargo test -p arcflow-core human_interrupt -- --ignored --nocapture
    cargo test -p arcflow-core trace_persistence -- --ignored --nocapture
  '
fi
if [[ "${CI_SKIP_STATIC_E2E:-}" == "1" || ! command -v docker &>/dev/null ]]; then
  echo ""; echo "=== static-e2e (skipped) ==="
else
  run_step "static-e2e" bash -c '
    docker compose -f docker/docker-compose.server.yml up -d --build
    for i in $(seq 1 60); do curl -sf http://localhost:8080/ready && break; sleep 2; done
    bash scripts/static-provision-site.sh | tee /tmp/site.env
    export SITE_ID="$(grep "^SITE_ID=" /tmp/site.env | cut -d= -f2-)" TEXT_FILE=examples/static/chat-rag/kb.txt
    bash scripts/static-ingest-knowledge.sh && bash scripts/static-publish-chat.sh
    pip install -q pytest requests && pytest examples/static/chat-rag/test_static.py -q
  '
fi
if [[ "${CI_SKIP_INTEGRATION_MEMORY:-}" == "1" || -z "${ARCFLOW_POSTGRESQL_URL:-}" || -z "${ARCFLOW_QDRANT_URL:-}" ]]; then
  echo ""; echo "=== integration-memory (skipped) ==="
else
  run_step "integration-memory" bash -c '
    cargo test -p arcflow-core sprint4_memory -- --test-threads=1
    cargo test -p arcflow-core sprint4_memory -- --ignored --test-threads=1
    cargo test -p arcflow-core sprint4_tools
    cd sdk-python && python -m pip install -q maturin pytest && python -m maturin develop
    tmpdir="$(mktemp -d)" && cd "$tmpdir"
    PYTHONPATH="" python -m pytest "$ROOT/sdk-python/tests/integration/test_memory_workflow.py::test_persistent_memory_survives_new_workflow_run" -v
  '
fi
if [[ "${CI_SKIP_SDK_PYTHON:-}" == "1" ]]; then
  echo ""; echo "=== sdk-python (skipped) ==="
else
  echo ""; echo "=== sdk-python ==="
  ( cd "$ROOT/sdk-python"
    python -m pip install -q maturin pytest "mypy<1.19" black ruff pytest-httpserver
    python -m maturin develop
    python -m ruff check arcflow tests && python -m black --check arcflow tests && python -m mypy arcflow
    tmpdir="$(mktemp -d)" && cd "$tmpdir"
    PYTHONPATH="" python -m pytest "$ROOT/sdk-python/tests" -v )
fi
echo ""; echo "ci-local-full: all requested steps passed"
