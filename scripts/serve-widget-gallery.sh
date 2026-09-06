#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
port="${1:-8081}"

cd "$repo_root"
# Relay permissions are deployment configuration, not per-site renderer exceptions.
relay_port="${ARGUI_WEBVIEW_RELAY_PORT:-$((port + 1))}"
export ARGUI_WEBVIEW_RELAY_URL="${ARGUI_WEBVIEW_RELAY_URL:-http://127.0.0.1:$relay_port/}"
relay_pid=""
cleanup() { if [[ -n "$relay_pid" ]]; then kill "$relay_pid" 2>/dev/null || true; wait "$relay_pid" 2>/dev/null || true; fi; }
trap cleanup EXIT
if [[ "$ARGUI_WEBVIEW_RELAY_URL" == "http://127.0.0.1:$relay_port/" ]]; then
    read -r -a relay_origins <<< "${ARGUI_WEBVIEW_ALLOWED_ORIGINS:-https://example.com}"
    relay_args=()
    for origin in "${relay_origins[@]}"; do relay_args+=(--allowed-origin "$origin"); done
    python3 crates/argui-webview/src/browser/relay/server.py \
        --app-origin "http://127.0.0.1:$port" --public-origin "$ARGUI_WEBVIEW_RELAY_URL" \
        --port "$relay_port" "${relay_args[@]}" &
    relay_pid=$!
fi
wasm-pack build crates/argui-widget-gallery --target web --dev --out-dir ../../web/widgets/pkg --features webview
if [[ -n "$relay_pid" ]] && ! kill -0 "$relay_pid" 2>/dev/null; then
    echo "WebView relay failed to start; check its origin configuration and port." >&2
    exit 1
fi
echo "Argui widget gallery: http://127.0.0.1:$port/widgets/"
python3 scripts/dev_server.py "$port" --directory web --entry /widgets/
