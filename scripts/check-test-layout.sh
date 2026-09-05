#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
failed=0

while IFS= read -r source_test_file; do
  echo "error: ${source_test_file#"$repo_root"/} is test code under src/" >&2
  failed=1
done < <(
  {
    find "$repo_root/crates" -type f -path '*/src/*' \
      \( -name 'tests.rs' -o -name '*_tests.rs' \) -print
    rg --files-with-matches --color never --pcre2 \
      '(?:#\s*\[\s*(?:(?:[A-Za-z_]\w*::)*test\b|wasm_bindgen_test\b|(?:cfg|cfg_attr)\s*\([^]]*\btest\b)|\bcfg!\s*\(\s*test\b)' \
      "$repo_root/crates" -g '**/src/**/*.rs' || true
  } | sort -u
)

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
