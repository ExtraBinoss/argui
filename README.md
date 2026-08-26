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
```
