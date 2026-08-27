# GPU effects

Effects keep the existing dependency direction:

```text
argui-ui              ergonomic style and builders
    |
argui-paint           declarative effects and layer commands
    |
argui-render          render graph, textures, pipelines, and WGPU execution

argui-effects         optional WGSL presets built on the public registration API
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
    expansion: f32,
    pixel_parameters: u32,
}
```

The display list uses validated, balanced `BeginLayer(LayerStyle)` and
`EndLayer` commands. A layer describes group opacity, blend mode, foreground
filters, backdrop filters, rounded masks, drop/inset shadows, and custom WGSL.
Logical geometry is converted to physical pixels once while lowering the render
graph, so DPI scaling is shared by native and WebGPU.
`pixel_parameter(index)` marks an opaque custom value that follows the same DPI
conversion; phases, ratios, colors, and other unitless values stay intact.

## Implemented rendering path

- Scenes without effects keep the direct surface path and allocate no offscreen
  texture.
- Nested layers lower to an explicit graph backed by a bounded 128 MiB reusable
  texture pool with observable allocation statistics.
- Each effect layer renders into conservative physical-pixel bounds, rounded
  outwards and intersected with its parent. Quad and glyph pipelines receive
  the target origin through dynamic uniforms, so cropped and full-surface
  rendering reuse the same prepared instances.
- Large blur radii use adaptive downsampling, a separable 13-tap Gaussian, and
  bilinear upsampling.
- Backdrops are recomposed with the original target outside their rounded mask;
  unrelated text therefore stays at full resolution.
- Drop shadows are masked outside the border box and inset shadows inside it.
- Brightness, contrast, saturation, hue rotation, opacity, color matrices,
  destination-aware blend modes, backdrop blur, and refraction share one GPU
  pipeline.
- Adjacent brightness, contrast, saturation, hue, opacity, and color-matrix
  operations compose into one affine 4x5 matrix pass. Blur, refraction, and
  custom shaders remain explicit ordering boundaries.

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

The wrapper supplies both textures, a linear sampler, viewport, target/source
regions and layer geometry, four values in `params.data`, and twenty more in
`params.matrix`. `uv` is local to the cropped target. Use
`global_pixel(uv)`—provided by the ABI wrapper—for coordinates in the complete
window. The wrapper also exposes `source_at(pixel)`, `backdrop_at(pixel)`, and
`layer_rounded_distance(pixel)` for generic spatial effects. Naga rejects
malformed WGSL. Missing shader IDs and parameter overflow return
structured errors rather than becoming silent no-ops.

Applications embed shaders at Rust compile time and declare them once on their
normal model. Native and WASM then use the same registration path; no shader
file or network request exists at runtime:

```rust
const FIRE: ShaderEffectId = ShaderEffectId(7);
const SHADERS: &[EffectShader] = &[
    EffectShader::new(FIRE, include_str!("border_fire.wgsl")),
];

impl UiApp for App {
    fn effect_shaders(&self) -> &'static [EffectShader] {
        SHADERS
    }

    // view/update omitted
}

let fire = Filter::Custom(
    CustomEffect::new(FIRE, [phase, 18.0])
        .pixel_parameter(1)
        .expansion(18.0),
);
```

WGSL still becomes a backend-specific GPU pipeline when the renderer starts;
that part cannot happen during the Rust build because native Vulkan/Metal/DX12
and browser WebGPU target different adapters.

`expansion` is declarative geometry, not shader guesswork. It enlarges the
conservative target and final composite mask, allowing outer glows and animated
border fire without allocating a viewport-sized layer texture. The shared
showcase uses this exact API.

## Optional preset crate

`argui-effects` is separate from the renderer, so an application pays for no
opinionated shader library unless it depends on and registers it:

```rust
use argui_effects::{AnimatedGradient, LiquidGlass, WorleyBorderFire};

fn effect_shaders(&self) -> &'static [EffectShader] {
    argui_effects::SHADERS
}

let layer = LayerStyle::new(bounds)
    .filter(WorleyBorderFire::new(animated_phase).filter())
    .backdrop(LiquidGlass::new().refraction(9.0).blur(3.5).filter());
```

Any element that can describe a layer can receive these filters. Their values
are ordinary `f32`s, so a `Timeline<f32>`, spring, decay, or transition can feed
them on each animation frame. The presets live under
`argui-effects/src/shaders/effects`; renderer-owned primitive shaders live under
`argui-render/src/shaders/primitives`.

## Primitive scopes

The UI layer can apply the same `LayerStyle` to a precise part of any element:

```rust
let title = Element::text("GPU text")
    .text_effect(
        LayerStyle::new(Default::default())
            .filter(AnimatedGradient::new(animated_phase).filter()),
    );

let button = button
    .background_effect(background_layer)
    .border_effect(border_layer)
    .content_effect(content_layer)
    .whole_effect(whole_element_layer);
```

The generic scopes are `WholeElement`, `Background`, `Border`, `Content`, and
`Text`. `Text` wraps glyph drawing only; selection and caret remain crisp and
independent. `Content` includes the element's text-input decorations and its
descendants but excludes its own background and border. Multiple layers in one
scope nest in declaration order.

Background and border normally remain one batched quad. They split into two
ordered primitives only when either has a scoped effect, so elements without
scoped effects keep the original allocation-free display-list path.

## Pixel tests and profiling

The renderer test suite validates built-in and wrapped custom WGSL with Naga.
It also renders a color-matrix pass into a headless WGPU texture and reads the
pixel back. The test exits explicitly when no headless adapter exists, so CI can
use software or hardware adapters without making non-GPU builders fail.

Profiling is opt-in and uses the same Rust callback on native and web:

```rust
let renderer = RendererConfig::default().profiling(true);

run_app(window, renderer, app, |event| {
    if let RuntimeEvent::RenderProfile(frame) = event {
        // frame.cpu_time, frame.draw_batches, frame.effects,
        // frame.texture_pool, frame.viewport_pixels, frame.direct_surface
    }
})?;
```

`EffectGraphStats::offscreen_pixels` exposes the conservative layer area, while
`TexturePoolStats` reports actual retained allocation and reuse. `cpu_time`
measures preparation and command encoding; GPU timestamp queries remain a
separate opt-in facility because browser support is adapter-dependent.

## Remaining measured work

1. Add optional GPU timestamp queries where the adapter exposes them.
2. Compare cropped-target memory and pass counts on representative native and
   browser workloads before adding specialized pipelines.
3. Add golden images for blur, rounded masks, refraction, and custom outer
   effects on the project CI adapters.

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
