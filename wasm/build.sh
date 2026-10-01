#!/usr/bin/env bash
# Compiles the crate to wasm32 and regenerates the JS bindings in pkg/.
set -euo pipefail
cd "$(dirname "$0")"

cargo build --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/wasm_demo.wasm \
  --out-dir pkg --target web

# The Go benchmark module, and the runtime glue it needs.
( cd gobench && GOOS=js GOARCH=wasm go build -o ../pkg/go_bench.wasm . )
cp "$(go env GOROOT)/lib/wasm/wasm_exec.js" pkg/

printf '\n%-20s %s\n' \
  wasm_demo_bg.wasm "$(du -h pkg/wasm_demo_bg.wasm | cut -f1)" \
  go_bench.wasm     "$(du -h pkg/go_bench.wasm     | cut -f1)"
