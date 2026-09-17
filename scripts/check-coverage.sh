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
boundary_regex='crates/argui-platform/src/file_picker/native\.rs|crates/argui-runtime/src/(multi\.rs|animation\.rs|app/(accessibility|frame|lifecycle|preferences|scroll|text_selection|window|popups|desktop_backdrop)\.rs|app/popups/input\.rs)|crates/argui-platform/src/desktop_backdrop/(linux(\.rs|/(wayland|x11)\.rs)|windows\.rs|macos\.rs)|crates/argui-platform/src/popup/(linux|windows|macos)\.rs|crates/argui-render/src/(text/|vector/|image/pipeline\.rs|surface/(configure|effects)\.rs)'

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
cleanup() {
  if [[ -z "${ARGUI_KEEP_COVERAGE_ARTIFACTS:-}" ]]; then
    cargo clean --target-dir "$coverage_target" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT
# Old feature/build variants contain duplicate coverage maps even after their
# raw profiles are removed. Keep dependency caches, but discard workspace maps.
CARGO_TARGET_DIR="$coverage_target" cargo "+$coverage_toolchain" llvm-cov clean --workspace
if [[ -n "${ARGUI_NATIVE_TESTS:-}" ]]; then
  # Run the GPU integration in isolation. The workspace pass below retains its profile and
  # produces the final report with the complete set of test binaries.
  CARGO_TARGET_DIR="$coverage_target" CARGO_INCREMENTAL=0 \
    CARGO_PROFILE_TEST_OPT_LEVEL=0 CARGO_PROFILE_TEST_DEBUG=0 \
    cargo "+$coverage_toolchain" llvm-cov nextest \
    --package argui-render --all-features --test surface --branch --no-report \
    --jobs 1 --run-ignored only
fi
echo "coverage: running workspace library and integration tests"
CARGO_TARGET_DIR="$coverage_target" CARGO_INCREMENTAL=0 \
  CARGO_PROFILE_TEST_OPT_LEVEL=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo "+$coverage_toolchain" llvm-cov nextest \
  --workspace --all-features --lib --tests --branch --no-clean \
  --ignore-filename-regex "$boundary_regex" \
  --jobs "$coverage_jobs" --status-level fail --final-status-level fail \
  --success-output never --failure-output immediate-final \
  --json --output-path "$report"

python3 "$repo_root/scripts/coverage-gate.py" "$report" "$minimum"
