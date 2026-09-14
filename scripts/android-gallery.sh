#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
android_project="$repo_root/mobile/android"
gradle_wrapper="$android_project/gradlew"
command_name="${1:-}"

if [[ -z "$command_name" ]]; then
  echo "usage: $0 {apk|install|aab|clean|devices|launch} [Gradle options...]" >&2
  exit 2
fi
shift

case "$command_name" in
  apk) gradle_task="assembleDebug" ;;
  install) gradle_task="installDebug" ;;
  aab) gradle_task="bundleRelease" ;;
  clean) gradle_task="clean" ;;
  devices)
    command -v adb >/dev/null || { echo "adb is missing; install Android platform-tools" >&2; exit 1; }
    exec adb devices "$@"
    ;;
  launch)
    command -v adb >/dev/null || { echo "adb is missing; install Android platform-tools" >&2; exit 1; }
    exec adb shell monkey -p dev.argui.widgetgallery.debug 1 "$@"
    ;;
  *)
    echo "unknown command: $command_name" >&2
    echo "usage: $0 {apk|install|aab|clean|devices|launch} [Gradle options...]" >&2
    exit 2
    ;;
esac

if [[ "$command_name" != clean ]]; then
  command -v cargo >/dev/null || { echo "cargo is missing; install Rust with rustup" >&2; exit 1; }
  cargo ndk --help >/dev/null 2>&1 || {
    echo "cargo-ndk is missing; install it with: cargo install cargo-ndk --locked" >&2
    exit 1
  }
fi

if [[ "$command_name" == install ]]; then
  command -v adb >/dev/null || { echo "adb is missing; install Android platform-tools" >&2; exit 1; }
fi

cd "$android_project"
"$gradle_wrapper" "$gradle_task" "$@"
