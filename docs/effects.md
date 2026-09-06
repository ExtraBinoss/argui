# Effects

Argui separates effect composition from effect libraries. `argui-render`
provides the typed registry, validates definitions and schedules passes;
`argui-effects` provides optional presets. Applications decide exactly which
definitions enter a renderer.

## Registering presets

Preset families are Cargo features and are disabled by default:

```toml
[dependencies]
argui-effects = { version = "0.1", features = ["artistic"] }
```

Build the registry once and pass it through renderer configuration:

```rust
let config = argui_render::RendererConfig::default()
    .effects(argui_effects::registry()?);
```

An effect used by the display list but absent from that registry is an explicit
rendering error. There is no process-global registration and no implicit preset
loading.

## Composable element operations

`Element::filter` appends a content filter; `backdrop_filter` appends an
independent background filter. Both preserve the same layer's other settings.
`mask` replaces its alpha mask and `opacity` sets clamped group opacity, not
per-primitive alpha. `transform` remains the geometric transform builder.
`clip(radii)` clips to rounded element bounds and sets the surface corner radii;
unlike an alpha mask, this geometric clip does not introduce an offscreen layer.
Calling `layer` explicitly replaces the complete layer configuration.

The pipeline remains layout → display list → layer analysis → necessary
offscreen passes → wgpu composition → surface. These builders do not register
effects or bundle optional shader presets. Gradient masks and additional effect
families are not implied by the current `LayerMask` variants.

## Defining an effect

Definitions use namespaced stable identifiers, a named typed parameter schema
and one or more fragment passes:

```rust
use argui_paint::EffectId;
use argui_render::{
    EffectDefinition, EffectParameter, EffectParameterType,
    EffectPassDefinition, EffectRegistry,
};

const TINT: EffectId = EffectId::new("acme.color.tint");
const PARAMETERS: &[EffectParameter] = &[
    EffectParameter::new("amount", EffectParameterType::F32),
];
const PASSES: &[EffectPassDefinition] = &[
    EffectPassDefinition::fragment("tint", r#"
fn argui_effect(
    _uv: vec2<f32>,
    source: vec4<f32>,
    _backdrop: vec4<f32>,
) -> vec4<f32> {
    let amount = argui_param_f32(0u);
    let tint = argui_srgb_to_linear(vec3<f32>(1.0, 0.2, 0.1));
    return mix(source, vec4<f32>(tint, source.a), amount);
}
"#),
];

let registry = EffectRegistry::new([
    EffectDefinition::new(TINT, PARAMETERS, PASSES),
])?;
```

The registry rejects duplicate identifiers, duplicate or empty schema names,
zero pass divisors and invalid WGSL. Each instance must provide the exact
parameter names and types in schema order.

## Instantiating an effect

```rust
use argui_paint::{EffectInstance, EffectValue, Filter};

let filter = Filter::Effect(EffectInstance::new(
    TINT,
    [("amount", EffectValue::F32(0.35))],
));
```

Supported values are scalar floats, signed and unsigned integers, booleans,
vectors, 3×3 and 4×4 matrices, colors and logical pixels. Values are packed into
adapter-bounded storage buffers; definitions are not given raw WGPU handles.

Passes may request a downsample divisor. The renderer combines it with the
explicit `EffectQuality` setting. `Normal` preserves full current quality;
`Balanced`, `Performance` and `Custom` are application choices.

`source`, `backdrop`, `source_at`, `backdrop_at`, and color parameters use
straight-alpha extended linear sRGB. The generated shader ABI unpremultiplies
sampled layer textures before invoking custom code and premultiplies its result
for the next renderer pass. Custom effects must not apply an sRGB transfer
function themselves. `argui_srgb_to_linear` is available for color literals
authored directly inside a shader.

## Liquid glass (opt-in)

Enable only `argui-effects`' `liquid-glass` feature to compile this preset; it is
independent of `artistic`, `blur` and `refraction`. Register
`argui_effects::registry()` in `RendererConfig::effects` as for other presets.

```rust
use argui_effects::LiquidGlass;

let glass = LiquidGlass::new()
    .refraction(12.0)
    .ior(1.45)
    .fresnel(0.12)
    .turbulence(0.0)
    .blur(0.8)
    .tint([0.15, 0.4, 1.0, 0.12]); // linear RGB and tint amount
let pane = pane.backdrop_filter(glass.filter());
```

The optical rim is adapted from [Liquid Glass Studio's WGSL shader](https://github.com/iyinchao/liquid-glass-studio/blob/d13c3e53813ebc1d8b52d878071b63212a550ebc/src/shaders-wgsl/fragment-main.wgsl),
MIT © 2024 Charles Yin. Its license is preserved in
`crates/argui-effects/LICENSE-liquid-glass-studio`. The adaptation retains its
curved-bezel incidence, Snell refraction, channel dispersion, fifth-power Fresnel
rim and angular/opposite-side glare. Argui supplies its own rounded SDF, bounded
logical-pixel displacement, premultiplied blur and linear-color composition;
it does not import Studio's DOM capture, viewport coordinates or LCH conversion.
The flat center stays undistorted. Optional seeded fractal noise is disabled by
default; `.frequency`, `.octaves` and `.seed` configure it when `.turbulence` is
nonzero. This is an adaptation, not Apple's proprietary implementation.
With `.backdrop_filter`, the source is the real scene snapshot behind the pane;
with `.filter`, it is the element's rendered content. Blur is optional and is
not what creates refraction. Apply a rounded `LayerMask` to contain the glass.

Frequency is measured per logical pixel and remains DPI-independent. Octaves
are bounded to 1–6; refraction to 0–64 logical pixels. Tint, saturation, edge
width, highlight and chromatic aberration are configurable. Invalid float inputs
are sanitized. The effect declares sampling expansion so the renderer captures
pixels outside the pane before displacing them. No animation loop runs while
the parameters and scene are unchanged.

`SelectionHost::backdrop_filter` and `TextSelectionToolbar::backdrop_filter`
accept any registered filter without making widgets depend on this preset.
The gallery includes a draggable comparison at the bottom of
**GPU effects / WGSL**. The selection menu uses a regular backdrop blur;
applications can explicitly choose another filter and registry.
The comparison uses the widget Slider for live refraction, blur, tint amount,
IOR, rim width, reflections, highlights, dispersion, saturation and noise.
Tint color is selected separately from its amount. Noise sliders are inactive
until noise is enabled; Reset restores the optics without moving the pane,
and Recenter brings the draggable pane back into view.

## Profiling passes

When renderer profiling and DevTools recording are active, every pass receives
a GPU timestamp label of the form `effect.<effect-id>.<pass-name>`. The trace
also contains pass start time, duration, processed pixels and the stable render
object identity, so custom effects participate in the same timeline and ranking
as built-in composition passes.
