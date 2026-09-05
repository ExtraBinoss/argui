#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
port="${1:-8081}"

cd "$repo_root"
wasm-pack build crates/argui-widget-gallery --target web --dev --out-dir ../../web/widgets/pkg
echo "Argui widget gallery: http://127.0.0.1:$port/widgets/"
python3 scripts/dev_server.py "$port" --directory web --entry /widgets/
