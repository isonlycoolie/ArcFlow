#!/usr/bin/env bash
# Build arcflow-wasm for wasm32 and generate JS bindings for Workers.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PKG="$ROOT/runtime/arcflow-wasm/pkg"
TARGET="wasm32-unknown-unknown"

cd "$ROOT"

if ! rustup target list --installed | grep -q "$TARGET"; then
  echo "Installing Rust target $TARGET..."
  rustup target add "$TARGET"
fi

echo "Building arcflow-wasm ($TARGET release)..."
cargo build -p arcflow-wasm --release --target "$TARGET"

WASM="$ROOT/target/$TARGET/release/arcflow_wasm.wasm"
if [[ ! -f "$WASM" ]]; then
  echo "ERROR: expected artifact at $WASM"
  exit 1
fi

if ! command -v wasm-bindgen >/dev/null 2>&1; then
  echo "ERROR: wasm-bindgen not found. Install with: cargo install wasm-bindgen-cli"
  exit 1
fi

mkdir -p "$PKG"
echo "Generating wasm-bindgen pkg/ ..."
wasm-bindgen "$WASM" --out-dir "$PKG" --target web --no-typescript

echo "Done. pkg/ at $PKG"
