# Argui example applications

These applications are product-shaped demonstrations rather than isolated
widget samples. They live in a separate Cargo workspace so normal Argui builds,
packages and dependency graphs do not include them.

| Application                              | What it demonstrates                                                                                                                          |
| ---------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------- |
| [Fake AI Harness](fake-ai-harness/)      | A bundled adaptation of Wikipedia's Large language model article streams 6,000 simulated tokens at 1,000 tokens/s through a responsive VList. |
| [Documentation examples](docs-examples/) | Seventeen exact-source applications compiled into the interactive learning site. Each guide displays the Rust module it actually runs.        |
| [GPU Canvas Lab](gpu-canvas/)             | A retained WGPU compute/render viewport with Argui controls, pan/zoom, pause, overlays, effects, diagnostics, native and WebAssembly builds.  |

Run an application from the repository root:

```sh
cargo run --manifest-path app_examples/Cargo.toml -p argui-example-ai-harness
```

GPU Canvas Lab has its own native and browser commands in its
[README](gpu-canvas/README.md).
