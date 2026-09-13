#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
mkdir -p target
gitleaks git --redact --log-opts=--all --report-format json \
  --report-path target/gitleaks-history.json
gitleaks git --redact --pre-commit --staged --report-format json \
  --report-path target/gitleaks-staged.json
