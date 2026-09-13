#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
command -v rg >/dev/null || {
  echo "error: ripgrep is required for source structure checks" >&2
  exit 1
}

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
echo "quality: model API compilation contracts"
cargo test -p argui-runtime --all-features --doc model::model_context::ModelContext
echo "quality: instrumented library coverage"
./scripts/check-coverage.sh
