#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT/sdk-typescript"

cargo build --release -p arcflow-node

OUT="$ROOT/sdk-typescript/prebuilds"
rm -rf "$OUT"
mkdir -p "$OUT"

copy_node() {
  local triple="$1"
  local src="$ROOT/target/release/arcflow_node.node"
  if [[ ! -f "$src" ]]; then
    src="$ROOT/target/release/libarcflow_node.so"
  fi
  if [[ ! -f "$src" ]]; then
    src="$ROOT/target/release/arcflow_node.dll"
  fi
  if [[ ! -f "$src" ]]; then
    echo "Native artifact not found after release build" >&2
    exit 1
  fi
  mkdir -p "$OUT/$triple"
  cp "$src" "$OUT/$triple/arcflow.node"
}

copy_node "linux-x64-gnu"
copy_node "darwin-x64"
copy_node "darwin-arm64"
copy_node "win32-x64-msvc"

echo "Prebuilt binaries written under sdk-typescript/prebuilds/"
