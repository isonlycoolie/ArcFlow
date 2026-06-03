#!/usr/bin/env bash
# Run CI smoke + Rust gates in Linux containers (GitHub Actions parity).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
if ! command -v docker &>/dev/null; then
  echo "ERROR: docker required for ci-docker.sh"
  exit 1
fi
docker_volume() {
  local dir="$1"
  case "$(uname -s)" in
    MINGW* | MSYS* | CYGWIN*)
      echo "${dir}:/work"
      ;;
    *)
      echo "${dir}:/work"
      ;;
  esac
}
VOL="$(docker_volume "$ROOT")"
run_docker() {
  if case "$(uname -s)" in MINGW* | MSYS* | CYGWIN*) true ;; *) false ;; esac; then
    MSYS_NO_PATHCONV=1 docker run --rm -v "$VOL" -w /work "$@"
  else
    docker run --rm -v "$VOL" -w /work "$@"
  fi
}
SMOKE_IMAGE="${CI_DOCKER_SMOKE_IMAGE:-ubuntu:24.04}"
RUST_IMAGE="${CI_DOCKER_RUST_IMAGE:-rust:bookworm}"
echo "=== ci-docker: smoke ($SMOKE_IMAGE) ==="
run_docker "$SMOKE_IMAGE" bash -c '
  set -euo pipefail
  export DEBIAN_FRONTEND=noninteractive
  apt-get update -qq
  apt-get install -y -qq git bash ca-certificates curl python3 nodejs npm >/dev/null
  bash scripts/ci-smoke.sh
'
echo "=== ci-docker: rust ($RUST_IMAGE) ==="
run_docker "$RUST_IMAGE" bash -c '
  set -euo pipefail
  export PATH="/usr/local/cargo/bin:$PATH"
  apt-get update -qq && apt-get install -y -qq git bash python3 >/dev/null
  rustup component add rustfmt clippy 2>/dev/null || true
  export CARGO_TERM_COLOR=always RUST_BACKTRACE=1
  cargo fmt --check
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test --workspace
  bash scripts/check-no-unwrap.sh
  bash scripts/check-no-sql-interpolation.sh
  bash scripts/check-function-length.sh
  python3 scripts/assert_provider_no_credentials.py
'
echo ""; echo "ci-docker: all steps passed"
