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
    .refraction(38.0)
    .edge_width(19.0)
    .depth_effect(false)
    .chromatic_aberration(0.0)
    .blur(8.0)
    .saturation(2.25)
    .brightness(0.05)
    .contrast(1.0)
    .highlight(0.5)
    .tint([0.0, 0.0, 0.0, 0.272]); // linear RGB and tint amount
let pane = pane.backdrop_filter(glass.filter());
```

The rounded lens is a WGSL port of
[Kyant's Backdrop 2.0.0 shaders](https://github.com/Kyant0/AndroidLiquidGlass/blob/bebb11a91bd97bf1dabde479f3b332ad9898731f/backdrop/src/commonMain/kotlin/com/kyant/backdrop/internal/Shaders.kt),
Apache-2.0 © 2025 Kyant. This is the library used by the linked
[SimpMusic LiquidGlass modifier](https://github.com/maxrave-dev/SimpMusic/blob/40006bab68d451bc5addc6759452d23b7252a8a2/composeApp/src/commonMain/kotlin/com/maxrave/simpmusic/expect/ui/LiquidGlass.kt).
The upstream license is preserved in
`crates/argui-effects/LICENSE-android-liquid-glass`; the shader header records
Argui's modifications. No SimpMusic application code is imported.

The port preserves the circular refraction profile, rounded-rectangle gradient,
optional radial depth, seven-sample spectral dispersion and directional rim
highlight. The refraction amount is inward, matching Backdrop's `Lens.kt`.
The center outside the rim stays undistorted; zero refraction or zero rim width
disables displacement. This replaces the previous Snell/IOR and fractal-noise
model; there are no IOR, Fresnel, frequency, seed or octave controls.

Argui adds two separable Gaussian blur passes (nine samples per axis over three
sigma), then the lens pass. Blur is optional and independent of refraction.
Color controls operate in Argui's linear color space, followed by the tint;
saturation 1, brightness 0 and contrast 1 preserve source colors.

Defaults follow SimpMusic's actual
[interactive glass container](https://github.com/maxrave-dev/SimpMusic/blob/40006bab68d451bc5addc6759452d23b7252a8a2/composeApp/src/commonMain/kotlin/com/maxrave/simpmusic/ui/component/LiquidGlassContainer.kt)
at neutral luminance (0.5), without a press: blur 8, brightness 0.05, contrast 1,
and saturation 2.25 (its vibrancy 1.5 followed by saturation 1.5). For the gallery's
76-pixel bar, the rim is one quarter of the height (19), refraction is half (38)
and radial depth is disabled. This avoids the radial discontinuity at the center.
The theme tint amount is 0.272, black in dark mode and white in light mode.
`LiquidGlass::new()` supplies the dark tint; the gallery resolves the theme color.
SimpMusic samples luminance over time; this demo starts at the documented neutral
setting and leaves every control editable. Blur and color-space processing use
Argui's renderer, so this is not a pixel-identical Android or Apple rendering.

Refraction and rim width are bounded to 0–64 logical pixels, blur sigma to
0–16. Dispersion is a dimensionless 0–1 amount; 1 matches Backdrop's enabled
setting. Saturation/contrast accept 0–4, brightness −1–1 and highlight/tint
amount 0–1. Invalid floats are sanitized. Lengths scale with display DPI;
colors, depth and dispersion do not. Sampling stays premultiplied through
blur and dispersion to preserve translucent edges. The declared expansion
includes maximum displacement, dispersion and three-sigma blur support.

With `.backdrop_filter`, the source is the real scene behind the pane;
with `.filter`, it is the element's rendered content. Apply a rounded
`LayerMask` to contain the glass. No animation loop runs while the scene and
parameters are unchanged. `SelectionHost::backdrop_filter` and
`TextSelectionToolbar::backdrop_filter` also accept this registered effect.

The gallery has an **Effects** category with separate **Liquid glass** and
**Scroll shadow** pages. Liquid glass shows a scrollable palette catalog and
fixed bottom navigation. Explore, Collections and Saved switch actual content;
colors scroll underneath the navigation's backdrop filter. Live controls
adjust the lens, blur, color, tint and depth, or disable the effect for comparison.
Reset restores the settings without changing the selected section or scroll.

## Profiling passes

When renderer profiling and DevTools recording are active, every pass receives
a GPU timestamp label of the form `effect.<effect-id>.<pass-name>`. The trace
also contains pass start time, duration, processed pixels and the stable render
object identity, so custom effects participate in the same timeline and ranking
as built-in composition passes.
