#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

echo "quality: source structure"
./scripts/check-rust-size.sh
./scripts/check-test-layout.sh
./scripts/check-source-hygiene.sh
echo "quality: formatting"
cargo fmt --all -- --check
echo "quality: native clippy"
cargo clippy --workspace --all-targets --all-features -- -D warnings
echo "quality: wasm check"
cargo check --workspace --all-targets --target wasm32-unknown-unknown
echo "quality: instrumented library coverage"
./scripts/check-coverage.sh
