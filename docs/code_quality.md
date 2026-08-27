# Code quality

`./scripts/quality.sh` is the local and CI acceptance gate. A change does not
pass unless every check succeeds.

## Hard limits

- Every tracked Rust file, including tests, examples, and build scripts, is at
  most 600 physical lines. Split a file when a responsibility becomes distinct;
  never split it merely to evade the limit.
- Workspace coverage must be at least 85% independently for lines, functions,
  LLVM regions, and branches.
- Formatting and Clippy warnings fail the check.
- `unsafe` is denied until a concrete, reviewed need is documented.

Coverage is a floor, not a reason to write low-value tests. Test public behavior,
edge cases, invalidation, event translation, layout results, and rendering data.
GPU image tests use deterministic off-screen targets and skip only when no
headless adapter is available.

`#[coverage(off)]` is reserved for the smallest OS/GPU callback or launch
boundary that requires a real display or graphics driver. Deterministic policy
extracted from that boundary must remain covered; broad exclusions fail review.

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
integration tests there automatically. Every integration-test path must mirror
an existing source path. Unit tests may stay beside small private algorithms.

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
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check --workspace --all-targets --target wasm32-unknown-unknown
./scripts/check-coverage.sh
```

Branch instrumentation currently requires Rust nightly. The application and all
normal checks stay on stable; only this measurement uses `cargo +nightly`.
