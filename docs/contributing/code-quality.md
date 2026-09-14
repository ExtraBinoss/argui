# Code quality

`./scripts/quality.sh` is the local and CI acceptance gate. A change does not
pass unless every check succeeds.

## Hard limits

- Every tracked Rust file, including tests, examples, and build scripts, is at
  most 600 physical lines. Split a file when a responsibility becomes distinct;
  never split it merely to evade the limit.
- Workspace coverage must be at least 85% independently for lines, functions,
  LLVM regions, and branches, both globally and for every workspace crate.
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

The project limits Cargo to six parallel build jobs in `.cargo/config.toml` and
Nextest to six simultaneous tests in `.config/nextest.toml`. Native graphical
coverage still uses one test worker to avoid competition on its private display.

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
artifacts live temporarily in `target/coverage/`; the coverage lock rejects
concurrent runs. Local gates delete those instrumented binaries after preserving
`target/coverage-report.json`. CI sets `ARGUI_KEEP_COVERAGE_ARTIFACTS=1` so its
Rust cache can reuse dependencies on the next run.
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

`ARGUI_NATIVE_TESTS=1` includes the opt-in native lifecycle, GTK input, WebView
and packaged application checks, plus the ignored renderer surface integration
test. The manual CPU profiling test remains excluded. Run this mode on the
private display described in [Linux graphical testing](linux-testing.md).
This mode defaults to one Nextest worker, since native windows and off-screen
renderer tests share the display/GPU. `ARGUI_COVERAGE_JOBS` explicitly overrides
that worker count. Native WebViews use the private display; no standalone browser
is launched.

The complete Linux gate includes these native checks on the private display:

```sh
./scripts/linux-hidden-display.sh env ARGUI_NATIVE_TESTS=1 ./scripts/quality.sh
```

Skipping native tests can leave native crates below the threshold; the gate
reports that failure instead of treating skipped behavior as covered.

The model-context compilation contracts live in
`crates/argui-runtime/tests/model/model_context.md` and are included in the public
API documentation. Rustdoc checks both rejected window capabilities and a valid
presentation consumer; Nextest remains the runner for behavioral tests.

The gate checks every package under `crates/`, including applications and crates
with no changes in the current work. Missing reports fail; a crate containing
only reexports has no instrumentable code and reports N/A. The JSON export and
per-crate gate results remain in `target/` for review.

## Dependencies and setup

The workspace [manifest](../../Cargo.toml) declares the Rust minimum and shared
dependency versions; [Cargo.lock](../../Cargo.lock) records the resolved graph.
Keep version numbers there instead of maintaining a second table in the docs.
Each crate declares only dependencies it uses. Target-specific and optional
integrations belong to their consumers' feature gates.

Winit/platform adapters belong to `argui-platform` and runtime hosts; WGPU to
`argui-render`; text shaping to `argui-text`. `argui-ui` uses Taffy style types,
while `argui-layout` owns the layout adapter. `argui-animation` and `argui-paint`
use only `argui-core` and the standard library. Applications supply their fonts;
the library does not force bundled font assets.

```sh
rustup target add wasm32-unknown-unknown
rustup toolchain install nightly --profile minimal --component llvm-tools-preview
cargo install cargo-llvm-cov --locked
cargo install cargo-nextest --locked
cargo fetch
```

The coverage JSON gate uses Python, with its tests under `tests/scripts`.
Optional Linux WebViews additionally need the development packages listed in
[WebView setup](../platform/webview.md#linux-build-dependencies). Graphical test prerequisites
are in [Linux testing](linux-testing.md).
