#!/usr/bin/env bash
# Needs: rustup target add wasm32-unknown-unknown  (opt-level 3 + wasm SIMD: about twice as fast as -Os)
set -euo pipefail
RUSTFLAGS="-C opt-level=3 -C target-feature=+simd128" cargo build --release --target wasm32-unknown-unknown
mkdir -p dist
# base64 is done in python (portable: no GNU `base64 -w0`, no huge argv)
python3 - <<'PY'
import base64
wasm = open("target/wasm32-unknown-unknown/release/turbocursa.wasm", "rb").read()
t = open("web/index.template.html").read().replace("__WASM_B64__", base64.b64encode(wasm).decode())
open("dist/turbocursa.html", "w").write(t)
print("dist/turbocursa.html ready (%d bytes)" % len(t))
PY
