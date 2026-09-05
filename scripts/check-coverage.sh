#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
summary="$(mktemp)"
coverage_lock="$repo_root/target/.argui-coverage-lock"
coverage_target="$repo_root/target/coverage"
minimum=90
coverage_jobs="${ARGUI_COVERAGE_JOBS:-6}"
boundary_regex='crates/argui-runtime/src/(multi\.rs|animation\.rs|app/(accessibility|frame|lifecycle|preferences|scroll|text_selection|window)\.rs)|crates/argui-render/src/(text/|vector/|image/pipeline\.rs|surface/(configure|effects)\.rs)'

command -v jq >/dev/null || {
  echo "error: jq is required to enforce all coverage metrics" >&2
  exit 1
}
command -v cargo-nextest >/dev/null || {
  echo "error: cargo-nextest is required to run the coverage suite" >&2
  exit 1
}

cd "$repo_root"
mkdir -p "$repo_root/target"
if ! mkdir "$coverage_lock" 2>/dev/null; then
  echo "error: another Argui coverage run is already active" >&2
  exit 1
fi
trap 'rm -f "$summary"; rmdir "$coverage_lock" 2>/dev/null || true' EXIT
# Old feature/build variants contain duplicate coverage maps even after their
# raw profiles are removed. Keep dependency caches, but discard workspace maps.
CARGO_TARGET_DIR="$coverage_target" cargo +nightly llvm-cov clean --workspace
echo "coverage: running workspace library and integration tests"
CARGO_TARGET_DIR="$coverage_target" CARGO_INCREMENTAL=0 \
  CARGO_PROFILE_TEST_OPT_LEVEL=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo +nightly llvm-cov nextest \
  --workspace --all-features --lib --tests --branch --no-clean \
  --ignore-filename-regex "$boundary_regex" \
  --jobs "$coverage_jobs" --status-level fail --final-status-level fail \
  --success-output never --failure-output immediate-final \
  --json --summary-only --output-path "$summary"

jq -r '.data[0].totals | ["branches", "functions", "lines", "regions"][] as $name |
  "coverage \($name): \(.[$name].percent | floor)%"' "$summary"

jq -e --argjson minimum "$minimum" '
  .data[0].totals as $totals |
  ["branches", "functions", "lines", "regions"] |
  all(.[]; $totals[.].percent >= $minimum)
' "$summary" >/dev/null || {
  echo "error: every coverage metric must be at least $minimum%" >&2
  exit 1
}
