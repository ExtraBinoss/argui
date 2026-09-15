# Code quality

`./scripts/quality.sh` is the acceptance gate. It checks source structure,
formatting, Clippy, WebAssembly, public API examples, and coverage.

## Rules

- Keep every tracked `.rs` file at 600 physical lines or fewer.
- Put tests in `tests/` and mirror the source path. Test code is forbidden
  under `src/`.
- Keep each crate focused and add a dependency only to the crate that uses it.
- Keep platform, UI, layout, text, paint, and renderer concerns separate.
- Deny unsafe code unless a concrete platform boundary has been reviewed and
  documented.
- Remove replaced APIs and callers together. Do not add migration shims,
  deprecated paths, or speculative abstractions.
- Do not commit `TODO`, `FIXME`, `todo!`, `unimplemented!`, or placeholder
  implementations.
- Document every new function and method with accurate Rustdoc. Explain
  parameters and return values; add `# Errors` and `# Panics` when applicable.
- Make ownership, dirty state, and cache bounds explicit. An idle UI must do no
  work.

Split files by responsibility, not solely to pass the line limit. Prefer plain
structs, enums, and short data flows over generic utility layers.

## Tests

Test behavior visible at a crate boundary: state transitions, invalidation,
event translation, layout output, paint data, accessibility, and error cases.
Do not expose private internals only for a test.

During implementation, run the smallest affected crate with the final feature
set so Cargo can reuse its artifacts:

```sh
cargo nextest run -p argui-ui --all-features
cargo clippy -p argui-ui --all-targets --all-features -- -D warnings
```

For a graphical change, follow [Linux graphical testing](linux-testing.md).
Tests must use the private display and inspect a saved capture; a blank capture
fails.

## Coverage

Each workspace crate and the workspace total must reach 85% independently for
lines, functions, LLVM regions, and branches. A crate with no instrumentable
code reports N/A. Missing reports fail.

Use `#[coverage(off)]` only on the smallest callback that requires a real OS
window or GPU. Extract deterministic policy from that callback and test it.
Renderer-neutral, layout, event, widget, and inspection code is never excluded.

Coverage uses nightly LLVM instrumentation and writes
`target/coverage-report.json`. It holds a lock because concurrent coverage
runs share instrumented artifacts.

```sh
rustup toolchain install nightly --profile minimal --component llvm-tools-preview
cargo install cargo-llvm-cov --locked
cargo install cargo-nextest --locked
./scripts/check-coverage.sh
```

Set `ARGUI_NATIVE_TESTS=1` to include native lifecycle, WebView, popup, input,
and renderer checks. Run that mode inside the private display.

## Final gate

Run the complete gate exactly once after implementation and immediately before
committing:

```sh
./scripts/linux-hidden-display.sh env ARGUI_NATIVE_TESTS=1 ./scripts/quality.sh
```

The script runs:

```text
source size and test-layout checks
cargo fmt --check
cargo clippy --workspace --all-targets --all-features
cargo check --workspace --all-targets --target wasm32-unknown-unknown
runtime public API doctest contracts
per-crate and workspace coverage
```

The workspace limits Cargo and Nextest concurrency in `.cargo/config.toml` and
`.config/nextest.toml`. Do not start another LLVM coverage run in parallel.

## Dependencies

The root [Cargo manifest](../../Cargo.toml) owns shared versions and the Rust
minimum; [Cargo.lock](../../Cargo.lock) owns the resolved graph. Put target
dependencies and feature flags in the consuming crate instead of documenting a
second version table.

Install the WebAssembly target before the first full check:

```sh
rustup target add wasm32-unknown-unknown
cargo fetch
```

WebView packages and Linux display prerequisites are documented in
[WebView](../platform/webview.md#linux-dependencies) and
[Linux graphical testing](linux-testing.md#prerequisites).
