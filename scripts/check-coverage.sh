#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
report="$repo_root/target/coverage-report.json"
coverage_lock="$repo_root/target/.argui-coverage.flock"
coverage_target="$repo_root/target/coverage"
minimum=85
coverage_toolchain="${ARGUI_COVERAGE_TOOLCHAIN:-nightly}"
coverage_jobs="${ARGUI_COVERAGE_JOBS:-6}"
if [[ -n "${ARGUI_NATIVE_TESTS:-}" && -z "${ARGUI_COVERAGE_JOBS:-}" ]]; then
  # The native lifecycle shares the display/GPU with off-screen renderer tests.
  coverage_jobs=1
fi
# GTK window dispatch is a platform boundary, like the other native window adapters below.
boundary_regex='crates/argui-platform/src/(file_picker/native|wayland_activation|wayland_global_shortcuts)\.rs|crates/argui-runtime/src/(multi\.rs|animation\.rs|app/(accessibility|frame|gtk|lifecycle|preferences|scroll|touch_scroll|text_selection|window|popups|desktop_backdrop)\.rs|app/popups/input\.rs)|crates/argui-platform/src/desktop_backdrop/(linux(\.rs|/(wayland|x11)\.rs)|windows\.rs|macos\.rs)|crates/argui-platform/src/popup/(linux|windows|macos)\.rs|crates/argui-render/src/(text/|vector/|image/pipeline\.rs|surface/(configure|effects)\.rs)'

command -v cargo-nextest >/dev/null || {
  echo "error: cargo-nextest is required to run the coverage suite" >&2
  exit 1
}
command -v flock >/dev/null || {
  echo "error: flock is required to serialize coverage runs" >&2
  exit 1
}

cd "$repo_root"
python3 -m unittest discover -s tests/scripts
mkdir -p "$repo_root/target"
exec 9>"$coverage_lock"
if ! flock -n 9; then
  echo "error: another Argui coverage run is already active" >&2
  exit 1
fi
# Remove prior workspace binaries and profile data before measuring a new run.
# Instrumented third-party dependencies stay cached in the dedicated target.
CARGO_TARGET_DIR="$coverage_target" cargo "+$coverage_toolchain" llvm-cov clean --workspace
if [[ -n "${ARGUI_NATIVE_TESTS:-}" ]]; then
  # The canvas stress test presents reliably on private X11. Run it in isolation;
  # the workspace pass below retains its profile for the combined report.
  ARGUI_TEST_BACKEND=x11 "$repo_root/scripts/linux-hidden-display.sh" env \
    ARGUI_SURFACE_STRESS=1 CARGO_TARGET_DIR="$coverage_target" CARGO_INCREMENTAL=0 \
    CARGO_PROFILE_TEST_OPT_LEVEL=0 CARGO_PROFILE_TEST_DEBUG=0 \
    cargo "+$coverage_toolchain" llvm-cov nextest \
    --package argui-render --all-features --test surface --branch --no-clean \
    --json --output-path "$repo_root/target/coverage-native-report.json" \
    --jobs 1 --run-ignored only
fi
echo "coverage: running workspace library and integration tests"
CARGO_TARGET_DIR="$coverage_target" CARGO_INCREMENTAL=0 \
  CARGO_PROFILE_TEST_OPT_LEVEL=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo "+$coverage_toolchain" llvm-cov nextest \
  --workspace --all-features --lib --tests --branch --no-clean \
  --ignore-filename-regex "$boundary_regex" \
  --jobs "$coverage_jobs" --status-level fail --final-status-level fail \
  --no-fail-fast --success-output never --failure-output immediate-final \
  --json --output-path "$report"

python3 "$repo_root/scripts/coverage-gate.py" "$report" "$minimum"
