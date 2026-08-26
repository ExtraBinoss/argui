#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
failed=0

while IFS= read -r -d '' test_file; do
  crate="${test_file%%/tests/*}"
  relative="${test_file#*/tests/}"
  source="$crate/src/$relative"

  if [[ ! -f "$source" ]]; then
    echo "error: ${test_file#"$repo_root"/} does not mirror ${source#"$repo_root"/}" >&2
    failed=1
  fi
done < <(find "$repo_root/crates" -type f -path '*/tests/*.rs' -print0)

exit "$failed"

