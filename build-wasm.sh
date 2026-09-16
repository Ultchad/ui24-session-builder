#!/usr/bin/env bash
set -euo pipefail

if ! command -v wasm-pack >/dev/null 2>&1; then
  echo "wasm-pack is required to build the browser bridge. Install it first:"
  echo "  cargo install wasm-pack"
  exit 1
fi

cargo build --target wasm32-unknown-unknown -p ui24_audio_processing
wasm-pack build --target web --release --out-dir "$PWD/applications/web/wasm" libraries/audio_processing
