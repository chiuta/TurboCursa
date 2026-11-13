#!/usr/bin/env bash
# Full check before a release: tests, lints, and every build target the project ships.
set -euo pipefail
cargo fmt --check
cargo test --release --no-fail-fast
cargo test --release --features web --test web_api --test web_code --test web_events
cargo clippy --release --all-targets --features native -- -D warnings
cargo build --release --features native
# the browser build must compile too (a platform gate removed by mistake only shows up here)
RUSTFLAGS="-C opt-level=3 -C target-feature=+simd128" cargo check --release --target wasm32-unknown-unknown
# nightly: cargo test --release --test fuzz_world -- --ignored
echo "all checks passed"
