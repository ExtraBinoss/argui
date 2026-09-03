# Argui — Another Rust GUI

An experimental, native-first Rust UI core built around `winit`, `wgpu`,
`cosmic-text`, and `taffy`. No WebView and no TypeScript frontend. Web support
means compiling the same renderer to WebAssembly/WebGPU.

The repository provides a retained UI tree, responsive layout, shaped text and
editing, interaction, scrolling, animation, transforms, gradients, decoded
images, scoped GPU effects, and one WGPU renderer for native and web. See the
[visual primitive contracts](docs/visual_primitives.md), the
[color contract](docs/color.md), and the
[overlay geometry API](docs/overlays.md), or the
[remaining roadmap](docs/roadmap.md).

```sh
cargo test --workspace
./scripts/quality.sh
cargo run -p argui --example window
cargo run -p argui --example text
cargo run -p argui --example layout
cargo run -p argui --example state
cargo run -p argui --example spotlight
cargo run -p argui-perf-showcase --example perf
./scripts/serve-web.sh
```

The native `state` example and the web build launch the same `StateShowcase`
Rust application from `argui-showcase`; only their tiny platform entrypoints
differ. Both wrap it in the optional `DevtoolsHost`, whose resizable Elements
and Profiling dock is itself composed from normal Argui widgets. Applications
that do not want inspection simply launch their `Render` component directly. Its fonts are
embedded because browsers do not expose system font files
to WASM. Argui itself ships no mandatory font; each application supplies the
assets and generic-family mapping it wants.

The `layout` example uses the retained `UiTree` API. Taffy computes logical
rectangles from the current viewport and Cosmic Text supplies intrinsic text
measurement, so native-window and browser resizing share the same reflow path.

The state showcase embeds a transparent PNG and a JPEG. `argui-image` is an
optional decoding boundary; the renderer receives validated RGBA assets and
does not depend on a file format or filesystem.
