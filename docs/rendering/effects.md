# GPU effects

`argui-render` validates effect definitions and executes their passes.
`argui-effects` provides optional presets. Applications register only the
definitions they use.

```toml
[dependencies]
argui-effects = { version = "0.3.0", features = ["artistic"] }
```

```rust,ignore
let renderer = argui_render::RendererConfig::default()
    .effects(argui_effects::registry()?);
```

Using an unregistered effect is a rendering error. Registration is explicit and
local to `RendererConfig`; there is no global registry.

## Element composition

- `filter` processes an element's rendered content.
- `backdrop_filter` processes the scene behind the element.
- `mask` sets the layer alpha mask.
- `opacity` sets group opacity.
- `clip` applies the element's rounded geometric clip without creating an
  offscreen layer by itself.
- `layer` replaces the complete layer configuration.

The renderer creates offscreen passes only when layer analysis requires them.
Filters do not change layout.

## Custom definition

An effect has a stable namespaced ID, typed parameter schema, and one or more
fragment passes:

```rust,ignore
use argui_paint::EffectId;
use argui_render::{
    EffectDefinition, EffectParameter, EffectParameterType,
    EffectPassDefinition, EffectRegistry,
};

const TINT: EffectId = EffectId::new("acme.color.tint");

let registry = EffectRegistry::new([EffectDefinition::new(
    TINT,
    &[EffectParameter::new("amount", EffectParameterType::F32)],
    &[EffectPassDefinition::fragment("tint", r#"
fn argui_effect(
    _uv: vec2<f32>,
    source: vec4<f32>,
    _backdrop: vec4<f32>,
) -> vec4<f32> {
    let amount = argui_param_f32(0u);
    let tint = argui_srgb_to_linear(vec3<f32>(1.0, 0.2, 0.1));
    return mix(source, vec4<f32>(tint, source.a), amount);
}
"#)],
)])?;
```

The registry rejects invalid WGSL, duplicate IDs or parameter names, empty
names, and invalid pass divisors. Instances provide exact names and types:

```rust,ignore
use argui_paint::{EffectInstance, EffectValue, Filter};

let filter = Filter::Effect(EffectInstance::new(
    TINT,
    [("amount", EffectValue::F32(0.35))],
));
```

Parameters support scalars, booleans, vectors, matrices, colors, and logical
pixels. The adapter packs them into bounded storage; custom definitions never
receive raw WGPU handles.

`source`, `backdrop`, sampling helpers, and color parameters use straight
alpha extended linear sRGB. The generated ABI unpremultiplies layer textures
before custom code and premultiplies the result afterward. Use
`argui_srgb_to_linear` for shader literals authored in sRGB.

Passes can request downsampling. `EffectQuality::{Normal, Balanced, Performance,
Custom}` lets the application combine that request with a global quality choice.

## Liquid glass

Enable the dedicated preset:

```toml
argui-effects = { version = "0.3.0", features = ["liquid-glass"] }
```

```rust,ignore
use argui_effects::LiquidGlass;

let glass = LiquidGlass::new()
    .refraction(38.0)
    .edge_width(19.0)
    .blur(8.0)
    .saturation(2.25)
    .brightness(0.05)
    .highlight(0.5)
    .tint([0.0, 0.0, 0.0, 0.272]);

let pane = pane.backdrop_filter(glass.filter());
```

The preset uses two separable Gaussian blur passes followed by the lens pass.
Refraction, rim, dispersion, color controls, tint, and optional radial depth are
bounded and sanitized. Logical lengths scale with DPI. No animation frame is
requested while the scene and parameters stay unchanged.

Use `backdrop_filter` for glass over existing content and `filter` to process
the element itself. Apply a rounded mask when the effect must stay inside a
specific shape.

The lens shader derives from Kyant's Apache-2.0 Backdrop implementation. The
source attribution, license, and Argui modifications are recorded in
`crates/argui-effects/LICENSE-android-liquid-glass` and the shader header.

## Profiling

With renderer profiling enabled, every pass records a timestamp named
`effect.<effect-id>.<pass-name>` plus duration, processed pixels, and render
object identity. This keeps custom effects visible in DevTools traces.

The Widget Gallery's **Effects** pages exercise presets, live parameters,
scrolling content, and fallback behavior.
