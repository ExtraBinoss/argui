#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 || ! "$1" =~ ^(desktop|android)$ || ! "$2" =~ ^(solid|react|minimal)$ ]]; then
  echo "usage: $0 {desktop|android} {solid|react|minimal}" >&2
  exit 2
fi

mode=$1
framework=$2
repo_root=$(cd "$(dirname "$0")/.." && pwd)
cd "$repo_root"
bun run generate:assets
if [[ "$framework" == react ]]; then
  bundle="$repo_root/apps/gallery/dist/gallery-react-core.mjs"
else
  bundle="$repo_root/apps/gallery/dist/gallery-core.mjs"
fi
ARGUI_GALLERY_DEV=1 ARGUI_GALLERY_ENTRY="$framework" bunx vite build --config apps/gallery/vite.config.ts --watch &
builder_pid=$!
sync_pid=
cleanup() {
  kill "$builder_pid" 2>/dev/null || true
  if [[ -n "$sync_pid" ]]; then kill "$sync_pid" 2>/dev/null || true; fi
}
trap cleanup EXIT

if [[ "$mode" == desktop ]]; then
  cargo_options=()
  if [[ "${ARGUI_GALLERY_ALL_TABLER:-0}" == 1 ]]; then cargo_options+=(--features dev-tabler-icons); fi
  CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$repo_root/target/dev}" ARGUI_GALLERY_BUNDLE="$bundle" \
    cargo run --manifest-path apps/gallery/quickjs-host/Cargo.toml --locked "${cargo_options[@]}"
else
  package=dev.argui.solidgallery.debug
  adb shell run-as "$package" mkdir -p files
  (
    previous=
    launched=0
    while kill -0 "$builder_pid" 2>/dev/null; do
      if [[ -f "$bundle" ]]; then
        signature=$(stat -c '%y:%s' "$bundle")
        if [[ "$signature" != "$previous" ]]; then
          adb shell run-as "$package" tee files/gallery-core.mjs.tmp < "$bundle" > /dev/null
          adb shell run-as "$package" mv files/gallery-core.mjs.tmp files/gallery-core.mjs
          previous=$signature
          echo "Pushed $framework bundle to Android"
          if [[ "$launched" == 0 ]]; then
            ./scripts/android-gallery.sh launch
            launched=1
          fi
        fi
      fi
      sleep 0.2
    done
  ) &
  sync_pid=$!
  wait "$builder_pid"
fi
