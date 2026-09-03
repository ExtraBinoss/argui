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

## Profiling

When renderer profiling and DevTools recording are active, every pass receives
a GPU timestamp label of the form `effect.<effect-id>.<pass-name>`. The trace
also contains pass start time, duration, processed pixels and the stable render
object identity, so custom effects participate in the same timeline and ranking
as built-in composition passes.
