# Argui example applications

These applications are product-shaped demonstrations rather than isolated
widget samples. They live in a separate Cargo workspace so normal Argui builds,
packages and dependency graphs do not include them.

| Application                              | What it demonstrates                                                                                                                          |
| ---------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------- |
| [Astra Editor](astra-editor/)            | A responsive, native/Web Rust editor with project navigation, multi-file tabs, virtualized search, resizable panels, and polished motion.     |
| [Fake AI Harness](fake-ai-harness/)      | A bundled adaptation of Wikipedia's Large language model article streams 6,000 simulated tokens at 1,000 tokens/s through a responsive VList. |
| [Documentation examples](docs-examples/) | Twenty-two exact-source applications compiled into the interactive learning site. Each guide displays the Rust module it actually runs.       |
| [GPU Canvas Lab](gpu-canvas/)            | A retained WGPU compute/render viewport with Argui controls, pan/zoom, pause, overlays, effects, diagnostics, native and WebAssembly builds.  |
| [Spotlight](spotlight/)                  | A translucent launcher with animated filtering, native desktop backdrop, tray lifecycle, and a global activation shortcut.                    |
| [Widget Gallery DSL](widget-gallery-dsl/) | Responsive DSL-authored component gallery with theme modes, virtual navigation, live reload, and bundled image/SVG assets.                |
| [DSL Live Demo](dsl-live-demo/)          | Small focused application for trying transactional `argui dev` updates.                                                                 |
| [VirtualList DSL](dsl-virtual-list/)     | Standalone large-data list sharing the gallery's reusable DSL ListView and row-template API.                                            |

Run an application from the repository root:

```sh
cargo run --manifest-path app_examples/Cargo.toml -p argui-example-ai-harness
```

GPU Canvas Lab has its own native and browser commands in its
[README](gpu-canvas/README.md).

To edit the DSL gallery with live reload:

```sh
cd app_examples/widget-gallery-dsl
cargo run --manifest-path ../../Cargo.toml -p argui-cli --bin argui -- dev
```

See the gallery's [README](widget-gallery-dsl/README.md) for release builds and
the component checklist.

Run the editor from the repository root:

```sh
cargo run --manifest-path app_examples/Cargo.toml -p argui-example-astra-editor
```

Run the Spotlight launcher with `Cmd/Ctrl+Space`:

```sh
cargo run --manifest-path app_examples/Cargo.toml -p argui-example-spotlight
```

On Wayland, approve `Ctrl+Space` in the desktop's first-run Global Shortcuts
dialog. The uninstalled example creates a hidden development `.desktop` entry
for that portal session and removes it on a clean exit; packaged applications
ship the matching entry normally.
