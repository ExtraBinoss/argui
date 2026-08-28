#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

failed=0
source_globs=(
  --glob '*.rs'
  --glob '*.toml'
)

if rg -n -i \
  '(^|[^[:alnum:]_])(legacy|deprecated)([^[:alnum:]_]|$)|backwards?[ -]compatib|compatib(ility|le)[ _-](layer|shim)|migration[ _-](layer|shim)|migrate[ _-](layer|shim)' \
  "${source_globs[@]}" crates Cargo.toml; then
  echo "error: migration, compatibility, deprecated, and legacy paths are forbidden" >&2
  failed=1
fi

if rg -n \
  '(^|[^[:alnum:]_])(TODO|FIXME|HACK|XXX|STUB|PLACEHOLDER)([^[:alnum:]_]|$)|todo!|unimplemented!' \
  "${source_globs[@]}" crates Cargo.toml; then
  echo "error: incomplete-code markers and placeholder implementations are forbidden" >&2
  failed=1
fi

if rg -n -U \
  'std::time::Instant|std::time::\{[^}]*\bInstant\b|std::\{[^}]*time::\{[^}]*\bInstant\b' \
  --glob '*.rs' crates; then
  echo "error: std::time::Instant panics on wasm32; use web_time::Instant" >&2
  failed=1
fi

while IFS= read -r -d '' path; do
  relative="${path#"$repo_root"/}"
  if [[ "$relative" =~ (^|/)(legacy|compat|compatibility|migration|migrations|shim)(/|\.|$) ]]; then
    echo "error: forbidden migration or compatibility path: $relative" >&2
    failed=1
  fi
done < <(find "$repo_root/crates" -print0)

exit "$failed"
