# GPU Canvas Lab

GPU Canvas Lab is a complete Argui application shell around one retained custom
WGPU viewport. The toolbar, inspector, focus handling, gestures, keyboard
commands, rounded clipping, effect layer, and status overlay are ordinary Argui
UI. The canvas callback uses Argui's exact WGPU re-export to run a bounded
256-particle compute pass and then a grid/particle render pass into the
Argui-owned offscreen target.

The example exercises the public API boundary directly. Its factory receives a
`GpuCanvasDeviceContext` and uses the exposed `wgpu::Device`, target format,
limits, and device generation to allocate buffers and pipelines. Every render
callback receives a `GpuCanvasRenderContext`, writes through its queue, and
encodes compute and render passes through its command encoder and target view.
The application deliberately never submits or presents; Argui owns those steps.

The model and renderer share a short-lived `RwLock` snapshot. Each pan, zoom,
or reset updates a camera destination, then animation ticks smoothly interpolate
the visible view and advance an explicit content revision. The callback copies
the snapshot, releases the lock, writes its uniform buffer, and encodes work.
While paused and with the camera settled, the model requests no animation
frames, the revision stays fixed, and unrelated Argui redraws reuse the retained
texture without invoking the callback.

Run the native release build from the repository root:

```sh
cargo run --release --manifest-path app_examples/Cargo.toml \
  -p argui-example-gpu-canvas
```

Check the WebAssembly target:

```sh
cargo check --manifest-path app_examples/Cargo.toml \
  -p argui-example-gpu-canvas --target wasm32-unknown-unknown
```

Build the browser package with the repository-pinned `wasm-pack` 0.15.0, copy
the generated package beside the supplied page, and serve it over HTTP:

```sh
wasm-pack build app_examples/gpu-canvas --target web --release \
  --out-dir web/pkg
python3 -m http.server --directory app_examples/gpu-canvas/web 8000
```

Open `http://localhost:8000` in a WebGPU-capable browser. Drag or use arrow keys
to pan; use the wheel, pinch, or `+`/`-` to zoom; press `0` to reset and Space to
pause/resume. Vertical dragging follows the pointer by default; “Drag Y” switches
between natural and inverted modes. “Test recovery” exercises the visible
placeholder and runtime diagnostic path; “Recover canvas” advances the revision
and retries.
