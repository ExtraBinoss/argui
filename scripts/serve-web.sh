#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
port="${1:-8080}"

cd "$repo_root"
wasm-pack build crates/argui-web-demo --target web --dev --out-dir ../../web/pkg
echo "Argui web demo: http://127.0.0.1:$port"
python3 -m http.server "$port" --directory web
