# Argui — Another Rust GUI

An experimental, native-first Rust UI core built around `winit`, `wgpu`,
`cosmic-text`, and `taffy`. No WebView and no TypeScript frontend. Web support
means compiling the same renderer to WebAssembly/WebGPU.

The repository currently provides the module boundaries, public UI tree, window
runtime, and a WGPU surface presenting on native and web. See
[the roadmap](docs/roadmap.md) for the next rendering layers.

```sh
cargo test --workspace
./scripts/quality.sh
cargo run -p argui --example window
cargo run -p argui --example text
cargo run -p argui --example layout
cargo run -p argui --example state
./scripts/serve-web.sh
```

The native `state` example and the web build launch the same `StateShowcase`
Rust application from `argui-showcase`; only their tiny platform entrypoints
differ. Its fonts are embedded because browsers do not expose system font files
to WASM. Argui itself ships no mandatory font; each application supplies the
assets and generic-family mapping it wants.

The `layout` example uses the retained `UiTree` API. Taffy computes logical
rectangles from the current viewport and Cosmic Text supplies intrinsic text
measurement, so native-window and browser resizing share the same reflow path.
