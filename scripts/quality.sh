#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
./scripts/check-quality-static.sh
echo "quality: instrumented library coverage"
./scripts/check-coverage.sh
