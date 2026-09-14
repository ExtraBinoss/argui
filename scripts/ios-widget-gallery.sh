#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
project_root="$repo_root/mobile/ios"
build_root="$project_root/build"
device_target="aarch64-apple-ios"
simulator_arm_target="aarch64-apple-ios-sim"
simulator_intel_target="x86_64-apple-ios"
library_name="libargui_widget_gallery.a"

usage() {
    cat <<'EOF'
Build the Argui Widget Gallery iOS artifacts.

Usage: scripts/ios-widget-gallery.sh COMMAND

Commands:
  check         Cross-check device and simulator Rust targets (works without Xcode)
  xcframework   Build a device + universal simulator XCFramework (macOS + Xcode)
  simulator    Build the unsigned iOS Simulator .app (macOS + Xcode)
  all          Build the XCFramework and the unsigned Simulator .app

Artifacts are written under mobile/ios/build/.
EOF
}

require_macos() {
    if [[ "$(uname -s)" != Darwin ]]; then
        echo "This command needs macOS and Xcode; use 'check' for Rust cross-checks on Linux." >&2
        exit 2
    fi
    if ! command -v xcodebuild >/dev/null 2>&1; then
        echo "xcodebuild is unavailable; install Xcode and its command-line tools." >&2
        exit 2
    fi
}

ensure_targets() {
    local target
    for target in "$@"; do
        if ! rustup target list --installed | grep -Fxq "$target"; then
            rustup target add "$target"
        fi
    done
}

check_targets() {
    ensure_targets "$device_target" "$simulator_arm_target" "$simulator_intel_target"
    local target
    for target in "$device_target" "$simulator_arm_target" "$simulator_intel_target"; do
        cargo check --locked --package argui-widget-gallery --lib --target "$target"
    done
}

build_libraries() {
    ensure_targets "$device_target" "$simulator_arm_target" "$simulator_intel_target"
    local target
    for target in "$device_target" "$simulator_arm_target" "$simulator_intel_target"; do
        cargo build --locked --release --package argui-widget-gallery --lib --target "$target"
    done
}

build_xcframework() {
    require_macos
    build_libraries
    mkdir -p "$build_root"

    local device_library="$repo_root/target/$device_target/release/$library_name"
    local arm_simulator_library="$repo_root/target/$simulator_arm_target/release/$library_name"
    local intel_simulator_library="$repo_root/target/$simulator_intel_target/release/$library_name"
    local simulator_library="$build_root/$library_name"
    local xcframework="$build_root/ArguiWidgetGallery.xcframework"

    local library
    for library in "$device_library" "$arm_simulator_library" "$intel_simulator_library"; do
        if [[ ! -f "$library" ]]; then
            echo "Expected Cargo static library was not produced: $library" >&2
            exit 1
        fi
    done

    xcrun lipo -create "$arm_simulator_library" "$intel_simulator_library" \
        -output "$simulator_library"
    rm -rf "$xcframework"
    xcodebuild -create-xcframework \
        -library "$device_library" \
        -headers "$project_root/include" \
        -library "$simulator_library" \
        -headers "$project_root/include" \
        -output "$xcframework"
}

build_simulator_app() {
    require_macos
    plutil -lint "$project_root/ArguiWidgetGallery.xcodeproj/project.pbxproj"
    build_xcframework
    xcodebuild \
        -project "$project_root/ArguiWidgetGallery.xcodeproj" \
        -scheme ArguiWidgetGallery \
        -configuration Debug \
        -sdk iphonesimulator \
        -destination 'generic/platform=iOS Simulator' \
        -derivedDataPath "$build_root/DerivedData" \
        CODE_SIGNING_ALLOWED=NO \
        CODE_SIGNING_REQUIRED=NO \
        build
}

command="${1:-}"
case "$command" in
    check)
        check_targets
        ;;
    xcframework)
        build_xcframework
        ;;
    simulator)
        build_simulator_app
        ;;
    all)
        build_simulator_app
        ;;
    -h|--help|help)
        usage
        ;;
    *)
        usage >&2
        exit 2
        ;;
esac
