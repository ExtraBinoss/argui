# GPU effects

Effects keep the existing dependency direction:

```text
argui-ui              ergonomic style and builders
    |
argui-paint           declarative effects and layer commands
    |
argui-render          render graph, textures, pipelines, and WGPU execution
```

`argui-paint` never imports WGPU, compiles shaders, or allocates textures. An UI
without effects keeps the current direct surface-rendering path and pays no
off-screen texture or extra-pass cost.

## Public model

The renderer-independent paint model is implemented around nested `LayerStyle`
values:

```rust
enum Filter {
    Blur(f32),
    Brightness(f32),
    Contrast(f32),
    Saturation(f32),
    HueRotate(f32),
    Opacity(f32),
    ColorMatrix([f32; 20]),
    Refraction(Refraction),
    Custom(CustomEffect),
}

enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Difference,
}

struct CustomEffect {
    shader: ShaderEffectId,
    parameters: Vec<f32>,
}
```

The display list uses validated, balanced `BeginLayer(LayerStyle)` and
`EndLayer` commands. A layer describes group opacity, blend mode, foreground
filters, backdrop filters, rounded masks, drop/inset shadows, and custom WGSL.
Logical geometry is converted to physical pixels once while lowering the render
graph, so DPI scaling is shared by native and WebGPU.

## Implemented rendering path

- Scenes without effects keep the direct surface path and allocate no offscreen
  texture.
- Nested layers lower to an explicit graph backed by a bounded 128 MiB reusable
  texture pool with observable allocation statistics.
- Large blur radii use adaptive downsampling, a separable 13-tap Gaussian, and
  bilinear upsampling.
- Backdrops are recomposed with the original target outside their rounded mask;
  unrelated text therefore stays at full resolution.
- Drop shadows are masked outside the border box and inset shadows inside it.
- Brightness, contrast, saturation, hue rotation, opacity, color matrices,
  destination-aware blend modes, backdrop blur, and refraction share one GPU
  pipeline.

## Custom WGSL ABI v1

`SurfaceRenderer::register_effect_shader` validates and caches source once. The
source defines only this function:

```wgsl
fn argui_effect(
    uv: vec2<f32>,
    source: vec4<f32>,
    backdrop: vec4<f32>,
) -> vec4<f32> {
    return mix(source, backdrop, params.data.x);
}
```

The wrapper supplies both textures, a linear sampler, viewport and layer
geometry, four values in `params.data`, and twenty more in `params.matrix`. Naga
rejects malformed WGSL. Missing shader IDs and parameter overflow return
structured errors rather than becoming silent no-ops.

## Follow-up optimization order

1. Crop offscreen targets to conservative layer bounds instead of viewport size.
2. Fuse adjacent color filters into one matrix pass.
3. Expose application-level shader registration and custom-effect expansion for
   outer effects such as animated border fire.
4. Add deterministic GPU pixel tests on CI adapters that support them.
5. Profile native and browser scenes before selecting further special cases.

## Acceptance checks for every step

- The same scene behaves on native WGPU and browser WebGPU.
- Paint-model and render-graph decisions have CPU tests without a GPU.
- Pixel tests use deterministic off-screen targets where supported.
- Nested layers, clipping, resize, DPI changes, and surface loss are covered.
- Idle UI performs no redraw and no effect work.
- A scene without effects uses no intermediate textures.
- Texture-pool memory is bounded and observable.
- All repository quality and coverage gates continue to pass.

The shared showcase exercises native/WASM backdrop glass, refraction, rounded
masking, and a continuously animated colored glow from one Rust UI tree.
