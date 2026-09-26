#!/usr/bin/env bash
set -euo pipefail

if [[ $# -gt 1 || ( $# -eq 1 && "$1" != quickjs ) ]]; then
  echo "Usage: $0 [quickjs]" >&2
  exit 2
fi

repo_root=$(cd "$(dirname "$0")/.." && pwd)
cd "$repo_root"
echo "[gallery:build] Generating JSX declarations and SDK snapshot…"
bun run generate:jsx
echo "[gallery:build] Building Solid and React gallery bundles…"
bun run build:gallery
echo "[gallery:build] Compiling the QuickJS host…"
cargo build --manifest-path apps/gallery/quickjs-host/Cargo.toml --locked
echo "[gallery:build] QuickJS host and gallery bundles are ready."
