# Code quality

`./scripts/quality.sh` is the local and CI acceptance gate. A change does not
pass unless every check succeeds.

## Hard limits

- Every tracked Rust file, including tests, examples, and build scripts, is at
  most 600 physical lines. Split a file when a responsibility becomes distinct;
  never split it merely to evade the limit.
- Workspace coverage must be at least 85% independently for lines, functions,
  LLVM regions, and branches, both globally and for every modified crate.
  Crate totals aggregate covered/count values, never file percentages. A metric
  with no instrumentable entries is N/A; a missing crate report fails the gate.
- Formatting and Clippy warnings fail the check.
- `unsafe` is denied until a concrete, reviewed need is documented.
- Migration shims, backward-compatibility layers, deprecated APIs, and legacy
  entry points are forbidden. Replace callers and keep one current path.
- Incomplete-code markers and stubs (`TODO`, `FIXME`, placeholder comments,
  `todo!`, or `unimplemented!`) are forbidden. A text input's user-visible
  placeholder is a real UI feature and is not an incomplete-code marker.

Coverage is a floor, not a reason to write low-value tests. Test public behavior,
edge cases, invalidation, event translation, layout results, and rendering data.
GPU image tests use deterministic off-screen targets and skip only when no
headless adapter is available.

`#[coverage(off)]` is reserved for the smallest OS/GPU callback or launch
boundary that requires a real display or graphics driver. Deterministic policy
extracted from that boundary must remain covered; broad exclusions fail review.
The coverage script keeps the same boundary as an explicit filename allowlist
for winit orchestration and WGPU resource code. This makes exclusions auditable
even when nightly branch export still reports regions annotated `coverage(off)`.
Renderer-neutral planning, layout, event, widget, and inspection code is never
part of that allowlist.

## Source and test structure

Use a responsibility folder only when it improves navigation:

```text
src/
  layout/
    grid.rs
tests/
  layout/
    grid.rs
```

The directory is named `tests/` rather than `test/` because Cargo discovers
integration tests there automatically. Every test path must mirror an existing
source path. Test functions, test-only modules, and test-only source files are
forbidden under `src/`; exercise private implementation through observable crate
behavior instead of exposing internals for tests.

Each crate owns one coherent responsibility and lists only the dependencies it
directly uses. Avoid generic `utils`, duplicate geometry types, hidden globals,
unbounded caches, and abstractions created for hypothetical future work.

## Implementation style

- Prefer plain structs, enums, and short explicit data flows.
- Reuse focused libraries for hard solved problems; keep adapters thin.
- Keep hot-path allocation visible. Reuse GPU buffers, glyph atlases, and
  temporary vectors only after measurement shows the need.
- Make retained state and dirty flags explicit. Do no work while the UI is idle.
- Separate platform events, UI state, layout, text shaping, and GPU painting.
- Profile CPU time, GPU time, allocations, and steady-state memory before
  optimizing.
- Add a dependency only with a documented owner and purpose.

## Commands

```sh
./scripts/check-rust-size.sh
./scripts/check-test-layout.sh
./scripts/check-source-hygiene.sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check --workspace --all-targets --target wasm32-unknown-unknown
cargo test -p argui-runtime --all-features --doc model::model_context::ModelContext
./scripts/check-coverage.sh
```

Branch instrumentation currently requires Rust nightly. The application and all
normal checks stay on stable; only this measurement uses `cargo +nightly`.
Coverage uses Nextest, disables incremental artifacts, and overrides the test
profile with `opt-level=0` and no debug symbols so LLVM measures Argui's source
branches without invalidating the stable interactive build cache. Instrumented
artifacts live in `target/coverage/`; the coverage lock rejects concurrent runs.
Before measuring, workspace instrumentation artifacts are cleaned while dependency
caches are retained, so old feature variants cannot add duplicate uncovered maps.

The gate reads the detailed JSON branch counters. When the source locations
account for all branches in a file, it unions each outcome across generic
instantiations; executing the true and false outcomes in different instantiations
covers both source outcomes. LLVM's original branch denominator is preserved.
For folded expressions or macro expansions whose locations do not account for
that denominator, LLVM's summary is retained. Functions, lines and regions use
LLVM's summaries. The aggregation and rejection threshold have regression tests:
`python3 -m unittest discover -s tests/scripts`.

`ARGUI_NATIVE_TESTS=1` includes the opt-in native lifecycle and GTK input checks.
This mode defaults to one Nextest worker, since native windows and off-screen
renderer tests share the display/GPU. `ARGUI_COVERAGE_JOBS` explicitly overrides
that worker count. It does not start a browser.

The model-context compilation contracts live in
`crates/argui-runtime/tests/model/model_context.md` and are included in the public
API documentation. Rustdoc checks both rejected window capabilities and a valid
presentation consumer; Nextest remains the runner for behavioral tests.

The coverage baseline defaults to the merge base with `origin/main`. Set
`ARGUI_COVERAGE_BASE` to the implementation's starting commit when work spans
multiple commits. The JSON export and gate results remain in `target/` for review.
