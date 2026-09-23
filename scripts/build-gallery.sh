#!/usr/bin/env bash
set -euo pipefail

if [[ $# -gt 1 || ( $# -eq 1 && "$1" != quickjs ) ]]; then
  echo "Usage: $0 [quickjs]" >&2
  exit 2
fi

repo_root=$(cd "$(dirname "$0")/.." && pwd)
cd "$repo_root"
bun run generate:jsx
bun run build:gallery
cargo build --manifest-path apps/gallery/quickjs-host/Cargo.toml --locked
echo "Built argui-gallery-quickjs from the shared Solid gallery"
