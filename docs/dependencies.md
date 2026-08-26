# Dependencies

Versions are pinned so a fresh checkout reproduces the same foundation. Renovate
or a deliberate maintenance change should update them one at a time.

| Crate | Version | Owner | Purpose |
|---|---:|---|---|
| `winit` | 0.30.13 | `argui-platform` | Native windows, web canvas lifecycle, events |
| `wgpu` | 30.0.1 | `argui-render` | Native GPU and WebGPU/WebGL backends |
| `taffy` | 0.14.0 | `argui-layout` | Flex, grid, and block layout algorithms |
| `cosmic-text` | 0.19.0 | `argui-text` | Shaping, bidi, fallback, line breaking, raster data |
| `pollster` | 1.0.1 | `argui-runtime` native | Resolve one-time GPU initialization |
| `wasm-bindgen-futures` | 0.4.77 | `argui-runtime` web | Non-blocking GPU initialization in the browser |
| `bytemuck` | 1.25.2 | `argui-render` | Checked POD data for GPU buffers |
| `thiserror` | 2.0.20 | planned for crates with public errors | Typed error boundaries |
| `tracing` | 0.1.44 | planned for the runtime | Opt-in diagnostics and profiling spans |

`glyphon` is deliberately absent for now: its release cadence can temporarily
target a different WGPU major. Argui will first build its own thin atlas/batching
layer from `cosmic-text` output, preventing two GPU stacks in one process. This
decision can be revisited with a measured prototype.

The web demo owns its Noto Sans and Fira Mono test assets and their OFL license
files. They are embedded at compile time to avoid browser font downloads and can
be replaced independently by every application.

Planned dependencies remain centralized in the workspace but are not attached to
a crate until code actually uses them. This keeps each crate manifest truthful.

`argui-paint` adds no third-party dependency. It contains plain display data;
only `argui-render` knows how WGPU turns that data into pixels.

`argui-ui` depends directly only on Argui's core, paint, and text descriptions.
Stable identity, hit testing, interaction state, and composed widgets do not
pull Winit, WGPU, or Taffy into the UI crate.

## Setup

Cargo installs library dependencies automatically:

```sh
rustup target add wasm32-unknown-unknown
rustup toolchain install nightly --profile minimal --component llvm-tools-preview
cargo install cargo-llvm-cov --locked
cargo fetch
```

The coverage gate also uses `jq` to validate the JSON summary because
`cargo-llvm-cov` has no native `--fail-under-branches` option yet.

Rust 1.89 is the workspace minimum because it is the highest current dependency
MSRV (`cosmic-text`). Development may use a newer stable compiler.
