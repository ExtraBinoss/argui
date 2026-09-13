# Visual primitives and color

Visual primitives stay renderer-independent. `argui-ui` exposes ergonomic
builders, `argui-paint` records immutable commands, and `argui-render` uploads
only visible frame data to WGPU. Native and WASM consume the same display list.

## Transforms

`Transform2D` provides translation, scale, rotation, skew, and a normalized
`TransformOrigin`. Transforms compose through the retained tree and apply to
quads, text, images, exact nested clips, effect layers, and hit testing. They
run after Taffy layout, so animating a transform never recomputes layout.

The type implements `Interpolate` and `MotionValue`. It therefore works with
typed keyframes, property-bound motions, springs, decay, and
velocity-preserving spring retargeting without a transform-specific timing
engine.

## Gradients

`Fill` is shared by every painted element. A linear or radial gradient can
therefore fill a container, button, card, or any composed widget:

```rust
let stops = GradientStops::from_vec(generated_stops)?;
let fill = Fill::Linear(LinearGradient::with_stops(
    Point::new(0.0, 0.0),
    Point::new(1.0, 0.0),
    stops,
))
```

There is no fixed per-gradient stop count. Stops are stored in shared immutable
memory on the CPU and packed into one storage buffer per frame. The total is
bounded by `RendererConfig::gradient_stop_capacity` (65,536 by default), so an
application can raise or lower the explicit GPU-memory/work budget. Gradient
geometry and individual stops can bind to typed motions without rebuilding the
element tree.

## Images

The renderer contract is a validated `ImageAsset`: stable `ImageId`, dimensions,
and RGBA8 pixels. The optional `argui-image` crate decodes PNG and JPEG with only
those two codec features enabled. Applications may instead supply pixels from
another decoder, an asset pack, or generated content.

`argui_image::ImageLibrary::insert` generates an opaque handle and stores the
decoded asset, so application code never invents numeric IDs. Applications
return the library assets once from `Render::image_assets`; both native and WASM
launchers register them automatically. `Element::image` supports
`Fill`, `Contain`, and `Cover`, linear or nearest sampling, rounded corners,
nested clipping, opacity, transforms, overlays, and effect layers.

GPU texture residency is bounded by `RendererConfig::image_cache_bytes` (64 MiB
by default). Registration fails explicitly if an asset cannot fit; normal
non-image scenes allocate no image texture.

## Showcase

Run `cargo run -p argui --example state` or `./scripts/serve-web.sh`. Both launch
the same Rust `StateShowcase`, including a generated 12-stop gradient, transformed
content, the supplied transparent PNG, and the supplied JPEG.

## Color

Argui has one color contract on native WGPU and WebGPU. Application colors are
authored in sRGB, converted once to extended linear sRGB, and remain linear
through painting and effects. An sRGB render target performs the final display
encoding. Display-P3, HDR, and ICC profiles are not part of this contract.

Use `Color::srgb`, `Color::srgba`, the 8-bit constructors, or `Color::from_hex`
for visual colors. `Color::linear_rgb` and `Color::linear_rgba` are explicit
low-level constructors for renderer math and shader data. `to_srgba` is for
serialization and editing; `to_linear_rgba` is for GPU uploads.

Primitive shaders output straight linear RGBA and normal WGPU blending creates
premultiplied linear intermediate textures. Built-in compositing, blur, shadows,
and blend modes preserve that representation. Custom `argui_effect` functions
receive and return straight linear RGBA; the generated ABI converts to and from
the renderer's premultiplied intermediate representation.

Gradients declare `ColorInterpolation::Oklab`, `LinearSrgb`, or `Srgb` at
construction. Stops are interpolated with premultiplied alpha to avoid dark
fringes, then converted to linear sRGB before rendering. Widget state colors and
typed color keyframes use OKLab interpolation by default.

Decoded image pixels and rasterized SVG atlases use sRGB textures. Glyph atlases
remain linear coverage masks and receive their linear text color in the shader.

## Vector rendering

`argui-vector` validates immutable SVG resources with `usvg` and retains their
resolution-independent source. Applications register the resulting
`VectorAsset` values through `Render::vector_assets`. `Element::vector(id)` then
emits a normal display-list primitive with `ImageFit`, paint-time color,
opacity, transforms, clips, z-order, layers and effects.

```rust
let mut vectors = VectorLibrary::new();
let id = vectors.insert_svg(include_bytes!("icon.svg"))?;

Element::vector(id)
    .vector_fit(ImageFit::Contain)
    .vector_color(theme.foreground)
    .width(Length::Px(24.0))
    .height(Length::Px(24.0));
```

### Raster and GPU boundary

`argui-render` parses each registered source into a retained `resvg` tree. The
first request for a physical-size variant rasterizes it with tiny-skia's
antialiased SVG renderer. That RGBA result is uploaded into a shared sRGB WGPU
atlas, initially 256×256. Later frames reuse resident variants without parsing
or rasterizing their SVG again.

The atlas grows from 256 KiB up to 16 MiB (2048×2048), with transparent two-pixel
gutters so linear sampling cannot leak adjacent icons. Nearby requested sizes
reuse the closest cached variant within explicit quality hysteresis. Growth
updates the texture binding and current frame's coordinates. At capacity, the
atlas can be recycled from visible content; a frame that cannot fit fails
explicitly. Memory stays bounded on native WGPU and WebGPU.

Adjacent vector commands share one render batch regardless of asset identity.
The common vector WGSL pipeline applies instance transforms, rounded clip-chain
coverage, opacity and tint without creating per-icon pipelines or bind groups.

### Color and static SVG features

SVGs authored with `currentColor` become alpha masks. Their RGB color is supplied
by `Element::vector_color`, so light/dark themes and interaction states update a
small instance record without duplicating or rerasterizing the asset. SVGs
without `currentColor` preserve their authored colors.

The retained static path accepts the shapes, curves, strokes, transforms,
gradients, opacity, clipping, masks and filters supported by the configured
`usvg`/`resvg` build. Font resolution, system fonts and external resource lookup
are intentionally not part of the icon renderer.

### Animation

Vector elements use the same paint-time transform and opacity bindings as other
primitives, so rotation, scale, translation and fades stay on the retained GPU
path. General SVG path morphing is not part of this API.

### Profiling

`RenderProfile::vector_atlas` reports entry count, cache hits, rasterizations and
allocated bytes for the current frame. DevTools records the same counters in
strict `argui-gpu-trace-v2` traces, making resize thrashing visible on Linux,
Windows, macOS and WebGPU.
