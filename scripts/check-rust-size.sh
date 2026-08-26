#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
limit=600
failed=0

while IFS= read -r -d '' file; do
  lines="$(wc -l < "$file")"
  if (( lines > limit )); then
    relative="${file#"$repo_root"/}"
    echo "error: $relative has $lines lines (maximum: $limit)" >&2
    failed=1
  fi
done < <(find "$repo_root" -type f -name '*.rs' \
  -not -path "$repo_root/target/*" -print0)

exit "$failed"

