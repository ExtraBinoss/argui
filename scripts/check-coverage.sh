#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
summary="$(mktemp)"
trap 'rm -f "$summary"' EXIT
minimum=85

command -v jq >/dev/null || {
  echo "error: jq is required to enforce all coverage metrics" >&2
  exit 1
}

cd "$repo_root"
cargo +nightly llvm-cov --workspace --all-features --all-targets --branch \
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
